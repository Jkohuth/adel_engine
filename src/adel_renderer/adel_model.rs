use std::sync::Arc;
use std::collections::HashMap;

use vulkano::{
    buffer::CpuAccessibleBuffer,
    device::Device,
};

use crate::adel_renderer::{
    renderer_utils,
    Vertex,
    VertexBuilder
};

#[derive(Debug)]
pub struct ModelBuilder {
    // May make these Option in the future
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl ModelBuilder {
    pub fn load_model(filepath: &str) -> ModelBuilder {
        let (models, materials) =
            tobj::load_obj(
                &filepath,
                &tobj::LoadOptions::default()
            )
            .expect("Failed to OBJ load file");
//        assert!(models[0].is_ok());
        //let mut unique_vertices = HashMap::new();
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for model in &models {
            for index in 0..&model.mesh.indices.len() /3 {
                let pos_offset = (3 * index) as usize;
                let norm_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;

                let mut vertex_builder: VertexBuilder = VertexBuilder::new();
                vertex_builder.position([model.mesh.positions[pos_offset   ],
                                         model.mesh.positions[pos_offset +1],
                                         model.mesh.positions[pos_offset +2]
                    ]
                );

                vertex_builder.normal(
                    [model.mesh.normals[norm_offset   ],
                     model.mesh.normals[norm_offset +1],
                     model.mesh.normals[norm_offset +2]]
                    );

                vertex_builder.uv([model.mesh.texcoords[tex_coord_offset   ] ,
                                   model.mesh.texcoords[tex_coord_offset +1] ]);

                let vertex = vertex_builder.build();
                /*if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex, index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }*/
            }
        }
        log::info!("Vertex Count: {}", vertices.len());

        Self {
            vertices,
            indices
        }
    }
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            vertices,
            indices,
        }
    }
    fn vertices(&mut self, vertices: Vec<Vertex>) -> &mut Self {
        self.vertices = vertices;
        self
    }

    fn indices(&mut self, indices: Vec<u32>) -> &mut Self {
        self.indices = indices;
        self
    }

    // Return the Model component with the Vertex and Index Buffers
    pub fn build(&self, device: &Arc<Device>) -> ModelComponent {
        ModelComponent {
            vertex_buffer: Some(renderer_utils::create_vertex_buffers(device, self.vertices.clone()).unwrap()),
            index_buffer: Some(renderer_utils::create_index_buffers(device, self.indices.clone()).unwrap())
        }
    }
}

// May need to update to include a Staging buffer in the future
#[derive(Debug)]
pub struct ModelComponent {
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u32]>>>,
}

impl ModelComponent {
    pub fn create_model_from_file(filepath: &str) -> ModelBuilder {
        ModelBuilder::load_model(filepath)
    }
    pub fn new(verticies: Vec<Vertex>, indicies: Vec<u32>) -> ModelBuilder {
        ModelBuilder::new(verticies, indicies)
    }
}

fn print_vertex(models: &tobj::Model) {
    let mesh = &models.mesh;
    println!("Indicies: {:?}", mesh.indices);
    println!("Position Count: {:?}", mesh.positions);
    println!("Normal Count: {:?}", mesh.normals);
    println!("Normal Indicies: {:?}", mesh.normal_indices);
    println!("Tex Count: {:?}", mesh.texcoords);
    println!("Tex Indices: {:?}", mesh.texcoord_indices);
    print!("\n\n");
    println!("Indicies: {:?}", mesh.indices.len());
    println!("Position Count: {:?}", mesh.positions.len());
    println!("Normal Count: {:?}", mesh.normals.len());
    println!("Normal Indicies: {:?}", mesh.normal_indices.len());
    println!("Tex Count: {:?}", mesh.texcoords.len());
    println!("Tex Indices: {:?}", mesh.texcoord_indices.len());
    print!("\n\n");
}