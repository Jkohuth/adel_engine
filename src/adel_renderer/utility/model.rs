use crate::adel_renderer::definitions::Vertex;
use crate::adel_renderer::utility::{
    buffers::AshBuffers, context::AshContext, descriptors::AshDescriptors,
};
use anyhow::Result;
use ash::vk;
use image::{DynamicImage, RgbaImage};
use nalgebra::{Vector2, Vector3};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use tobj;

pub struct ModelComponent {
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub indices_count: u32,

    //pub uniform_buffers: Vec<vk::Buffer>,
    //pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub texture_image: Option<vk::Image>,
    pub texture_image_memory: Option<vk::DeviceMemory>,
    pub texture_image_view: Option<vk::ImageView>,
    pub texture_sampler: Option<vk::Sampler>,
}

impl ModelComponent {
    pub fn builder() -> ModelComponentBuilder {
        ModelComponentBuilder::new()
    }

    pub fn destroy_model_component(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.vertex_buffer, None);
            device.free_memory(self.vertex_buffer_memory, None);
            device.destroy_buffer(self.index_buffer, None);
            device.free_memory(self.index_buffer_memory, None);
            match self.texture_image {
                Some(texture_image) => {
                    device.destroy_image(self.texture_image.unwrap(), None);
                    device.free_memory(self.texture_image_memory.unwrap(), None);
                    device.destroy_image_view(self.texture_image_view.unwrap(), None);
                    device.destroy_sampler(self.texture_sampler.unwrap(), None);
                }
                None => {}
            }

            //for i in self.uniform_buffers.iter().enumerate() {
            //    device.destroy_buffer(self.uniform_buffers[i.0], None);
            //    device.free_memory(self.uniform_buffers_memory[i.0], None);
            //}
        }
    }
}

pub struct ModelComponentBuilder {
    vertices: Option<Vec<Vertex>>,
    indices: Option<Vec<u32>>,
    image_object: Option<DynamicImage>,
    image_rgba: Option<RgbaImage>,
    image_size: vk::DeviceSize,
    image_width: u32,
    image_height: u32,
}

// FIXME: Setup the builder class properly
impl ModelComponentBuilder {
    pub fn new() -> Self {
        Self {
            vertices: None,
            indices: None,
            image_object: None,
            image_rgba: None,
            image_size: 0,
            image_width: 0,
            image_height: 0,
        }
    }
    pub fn load_model(mut self, file_path: &Path) -> Self {
        let mut reader = BufReader::new(File::open(file_path).expect("Faild to open File"));

        let mut indices: Vec<u32> = Vec::new();
        let mut vertices: Vec<Vertex> = Vec::new();

        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )
        .unwrap();

        let mut unique_vertices = HashMap::new();
        for model in &models {
            // Position
            for (i, index) in model.mesh.indices.iter().enumerate() {
                let pos_offset = (3 * index) as usize;
                let normal_offset = (3 * model.mesh.normal_indices[i]) as usize;
                let uv_offset = (2 * model.mesh.texcoord_indices[i]) as usize;

                let mut vertex_builder = Vertex::builder()
                    .position(Vector3::new(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ))
                    .normal(Vector3::new(
                        model.mesh.normals[normal_offset],
                        model.mesh.normals[normal_offset + 1],
                        model.mesh.normals[normal_offset + 2],
                    ))
                    .uv(Vector2::new(
                        model.mesh.texcoords[uv_offset],
                        model.mesh.texcoords[uv_offset + 1],
                    ));

                // Confirm if Vertex Colors were supplied for this Model, if not builder will set them to default
                if model.mesh.vertex_color.len() > 0 {
                    let color_offset = (3 * index) as usize;
                    vertex_builder = vertex_builder.color(Vector3::new(
                        model.mesh.vertex_color[color_offset + 0],
                        model.mesh.vertex_color[color_offset + 1],
                        model.mesh.vertex_color[color_offset + 2],
                    ));
                } else {
                    vertex_builder = vertex_builder.color(Vector3::new(0.7, 0.7, 0.7));
                }

                let vertex = vertex_builder.build();

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex.clone(), index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }
            }
        }
        self.vertices = Some(vertices);
        self.indices = Some(indices);
        self
    }
    pub fn load_texture(mut self, image_path: &Path) -> Self {
        let mut image_object: DynamicImage = image::open(image_path).unwrap();
        image_object = image_object.flipv();
        let (image_width, image_height) = (image_object.width(), image_object.height());
        // Size is u8 - per color size, 4 - rgba, width*height - area
        let image_size =
            (std::mem::size_of::<u8>() as u32 * image_width * image_height * 4) as vk::DeviceSize;
        // This crushes 16/32 bit pixel definition to 8 bit
        let image_rgba = image_object.clone().into_rgba8();

        self.image_object = Some(image_object);
        self.image_rgba = Some(image_rgba);
        self.image_size = image_size;
        self.image_width = image_width;
        self.image_height = image_height;
        self
    }
    // TODO: Build requires arguments in order to create the Result, additionally it doesn't consume the builder itself
    // as doing so would cause issues with the ModelComponentBuilder World Component
    pub fn build(
        &self,
        context: &AshContext,
        device: &ash::Device,
        buffers: &AshBuffers,
    ) -> Result<ModelComponent> {
        let (vertex_buffer, vertex_buffer_memory) = AshBuffers::create_vertex_buffer(
            context,
            device,
            self.vertices.as_ref().unwrap(),
            buffers.command_pool(),
            buffers.submit_queue(),
        )?;
        let (index_buffer, index_buffer_memory) = AshBuffers::create_index_buffer(
            context,
            device,
            self.indices.as_ref().unwrap(),
            buffers.command_pool(),
            buffers.submit_queue(),
        )?;

        let texture_image;
        let texture_image_memory;
        let texture_image_view;
        let texture_sampler;
        match &self.image_rgba {
            Some(_image_rgba) => {
                let texture_image_tuple = AshBuffers::create_texture_image(
                    context,
                    device,
                    self.image_width,
                    self.image_height,
                    self.image_size,
                    self.image_rgba.clone().unwrap(),
                    buffers.command_pool(),
                    buffers.submit_queue(),
                )?;
                texture_image = Some(texture_image_tuple.0);
                texture_image_memory = Some(texture_image_tuple.1);
                texture_image_view = Some(AshBuffers::create_texture_image_view(
                    device,
                    texture_image.unwrap(),
                )?);
                texture_sampler = Some(AshBuffers::create_texture_sample(device)?);
            }

            None => {
                texture_image = None;
                texture_image_memory = None;
                texture_image_view = None;
                texture_sampler = None;
            }
        }

        Ok(ModelComponent {
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            indices_count: self.indices.as_ref().unwrap().len() as u32,
            texture_image,
            texture_image_memory,
            texture_image_view,
            texture_sampler,
        })
    }
}
