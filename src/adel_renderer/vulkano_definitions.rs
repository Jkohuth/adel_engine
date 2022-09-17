use std::sync::Arc;
use bytemuck::{Zeroable, Pod,};
use glam::{Vec2, Vec3, Vec4, Mat2, Mat4};
use nalgebra::Vector3;
use log;
//use cgmath::{ BaseFloat, Matrix2, Matrix4, Rad, SquareMatrix, Vector2, Vector3, Vector4 };

use vulkano::{
    buffer::CpuAccessibleBuffer,
    device::Device,
    image::{
        swapchain::SwapchainImage, StorageImage,view::ImageView,
    },
};
use winit::window::Window;

use crate::adel_renderer::renderer_utils;

// Final render target onto which the whole app is rendered (per window)
pub type FinalImageView = Arc<ImageView<SwapchainImage<Window>>>;
/// Multipurpose image view
pub type DeviceImageView = Arc<ImageView<StorageImage>>;


<<<<<<< Updated upstream
=======
pub struct VertexBuilder {
    position: Option<[f32;3]>,
    color: Option<[f32;3]>,
    normal: Option<[f32;3]>,
    uv: Option<[f32;2]>
}
impl VertexBuilder {
    pub fn new() -> VertexBuilder {
        Self {
            position: None,
            color: None,
            normal: None,
            uv: None,
        }
    }
    pub fn position(&mut self, position: [f32; 3]) -> &mut Self {
        self.position = Some(position);
        self
    }
    pub fn color(&mut self, color: [f32; 3]) -> &mut Self {
        self.color = Some(color);
        self
    }

    pub fn normal(&mut self, normal: [f32; 3]) -> &mut Self {
        self.normal = Some(normal);
        self
    }
    pub fn uv(&mut self, uv: [f32; 2]) -> &mut Self {
        self.uv = Some(uv);
        self
    }

    pub fn build(&self) -> Vertex {
        Vertex {
            position: self.position.unwrap_or_default(),
            color: self.color.unwrap_or_default(),
            normal: self.normal.unwrap_or_default(),
            uv: self.uv.unwrap_or_default(),
        }
    }
}


>>>>>>> Stashed changes
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
<<<<<<< Updated upstream
=======
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn new() -> VertexBuilder {
        VertexBuilder::new()
    }
>>>>>>> Stashed changes
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex2d {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position, color);
vulkano::impl_vertex!(Vertex2d, position, color);


#[derive(Debug)]
pub struct TransformComponent {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Vec3,
}

impl TransformComponent {
    pub fn new(translation: Vec3, scale: Vec3, rotation: Vec3) -> Self {
        Self {
            translation,
            scale,
            rotation,
        }
    }

    // Matrix is translate * Ry * Rx * Rz * Sx * Sy * Sz
    pub fn mat4(&self) -> Mat4 {
        let mut transform = Mat4::from_translation(self.translation);
        transform = transform * Mat4::from_rotation_y(self.rotation.y);
        transform = transform * Mat4::from_rotation_x(self.rotation.x);
        transform = transform * Mat4::from_rotation_z(self.rotation.z);
        // TODO: Right now scale only works as a scalar (shocking) do non-scalar scaling
        transform = transform * Mat4::from_scale(self.scale);
        transform
    }
    pub fn mat4_less_computation(&self) -> Mat4 {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        Mat4 {
            x_axis: Vec4::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                self.scale.x * (c2 * s3),                // 01
                self.scale.x * (c1 * s2 * s3 - c3 * s1), // 02
                0.0),                                    // 03
            y_axis: Vec4::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3), // 10
                self.scale.y * (c2 * c3),                // 11
                self.scale.y * (c1 * c3 * s2 + s1 * s3), // 12
                0.0),                                    // 13
            z_axis: Vec4::new(
                self.scale.z * (c2 * s1),                // 20
                self.scale.z * (-s2),                    // 21
                self.scale.z * (c1 * c2),                // 22
                0.0),                                    // 23
            w_axis: Vec4::new(
                self.translation.x,                      // 30
                self.translation.y,                      // 31
                self.translation.z,                      // 32
                1.0)                                     // 33
        }

    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vec3 = Vec3::new(1.0, 1.0, 1.0);
        // No rotation for default
        let rotation: Vec3 = Vec3::new(0.0, 0.0, 0.0);
        Self {
            translation,
            scale,
            rotation,
        }
    }
}

#[derive(Debug)]
pub struct Transform2dComponent {
    pub translation: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Transform2dComponent {
    pub fn new(translation: Vec2, scale: Vec2, rotation: f32) -> Self {
        Self {
            translation,
            scale,
            rotation
        }
    }

    pub fn mat2(&self) -> Mat2 {
        let sin: f32 = self.rotation.sin();
        let cos: f32 = self.rotation.cos();
        // This may be wrong, I haven't used 2D rendering with glam
        let rot_mat = Mat2::from_cols(
            Vec2::new(cos, -sin), // 00 10
            Vec2::new(sin, cos)  // 10 11
        );
        let scale_mat = Mat2::from_cols(
            Vec2::new(self.scale.x, 0.0),
            Vec2::new(0.0, self.scale.y)
        );
        /* {
            x_axis: Vec2::new(0.0, 0.0),
            y_axis: Vec2::new(0.0, 0.0),
        }
        let rot_mat = Matrix2::<f32>::new(cos, sin, -sin, cos);
        let scale_mat = Matrix2::<f32>::new(self.scale.x, 0.0, 0.0, self.scale.y);*/
        rot_mat * scale_mat
    }
}
impl Default for Transform2dComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vec2 = Vec2::new(0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vec2 = Vec2::new(1.0, 1.0);
        // No rotation for default
        let rotation: f32 = 0.0;
        Self {
            translation,
            scale,
            rotation,
        }
    }
}

<<<<<<< Updated upstream
#[derive(Debug)]
pub struct ModelBuilder {
    // May make these Option in the future
    verticies: Vec<Vertex>,
    indicies: Vec<u16>,
}

impl ModelBuilder {
    pub fn new(verticies: Vec<Vertex>, indicies: Vec<u16>) -> Self {
        Self {
            verticies,
            indicies,
        }
    }
    // Return a tuple of a vertex and index buffer
    pub fn build(&self, device: &Arc<Device>) -> (Arc<CpuAccessibleBuffer<[Vertex]>>, Arc<CpuAccessibleBuffer<[u16]>>) {
            (renderer_utils::create_vertex_buffers(device, self.verticies.clone()).unwrap(),
             renderer_utils::create_index_buffers(device, self.indicies.clone()).unwrap() )
    }
}

// May need to update to include a Staging buffer in the future
#[derive(Debug)]
pub struct ModelComponent {
    pub builder: ModelBuilder,
    pub vertex_buffer: Option<Arc<CpuAccessibleBuffer<[Vertex]>>>,
    pub index_buffer: Option<Arc<CpuAccessibleBuffer<[u16]>>>,
}

impl ModelComponent {
    pub fn new(builder: ModelBuilder) -> Self {
        Self {
            builder,
            vertex_buffer: None,
            index_buffer: None,
        }
    }
    pub fn build(&mut self, device: &Arc<Device>) {
        // If Vertex_buffer exists and this was called again just pass, hmmmmmm maybe log?
        match self.vertex_buffer {
            None => {
                let buffers = self.builder.build(device);
                self.vertex_buffer = Some(buffers.0);
                self.index_buffer = Some(buffers.1);
            }
            _ => {
                log::debug!("Build called on object that already has buffer");
            }
        }
    }
}

=======
>>>>>>> Stashed changes

#[derive(Debug)]
pub struct TriangleComponent {
    pub verticies: Vec<Vertex2d>,

}

impl TriangleComponent {
    pub fn new(verticies: Vec<Vertex2d>) -> Self {
        assert_eq!(verticies.len(), 3);
        Self {
            verticies,
        }
    }
}