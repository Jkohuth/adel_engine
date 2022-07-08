use std::sync::Arc;
use bytemuck::{Zeroable, Pod,};

use cgmath::{ BaseFloat, Matrix2, Matrix4, Rad, SquareMatrix, Vector2, Vector3, Vector4 };

use vulkano::{
    image::{
        swapchain::SwapchainImage, StorageImage,view::ImageView,
    },
};
use winit::window::Window;

// Final render target onto which the whole app is rendered (per window)
pub type FinalImageView = Arc<ImageView<SwapchainImage<Window>>>;
/// Multipurpose image view
pub type DeviceImageView = Arc<ImageView<StorageImage>>;


#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
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
    pub translation: Vector3::<f32>,
    pub scale: Vector3::<f32>,
    pub rotation: Vector3::<Rad<f32>>,
}

impl TransformComponent {
    pub fn new(translation: Vector3::<f32>, scale: Vector3::<f32>, rotation: Vector3::<Rad<f32>>) -> Self {
        Self {
            translation,
            scale,
            rotation,
        }
    }

    // Matrix is translate * Ry * Rx * Rz * Sx * Sy * Sz
    pub fn mat4(&self) -> Matrix4::<f32> {
        let mut transform = Matrix4::from_translation(self.translation);
        transform = transform * Matrix4::from_angle_y(self.rotation.y);
        transform = transform * Matrix4::from_angle_x(self.rotation.x);
        transform = transform * Matrix4::from_angle_z(self.rotation.z);
        // TODO: Right now scale only works as a scalar (shocking) do non-scalar scaling
        transform = transform * Matrix4::from_scale(self.scale.x);
        transform
    }
    pub fn mat4_less_computation(&self) -> Matrix4::<f32> {
        let c3 = self.rotation.z.0.cos();
        let s3 = self.rotation.z.0.sin();
        let c2 = self.rotation.x.0.cos();
        let s2 = self.rotation.x.0.sin();
        let c1 = self.rotation.y.0.cos();
        let s1 = self.rotation.y.0.sin();
        Matrix4::<f32> {
            x: Vector4::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3),
                self.scale.x * (c2 * s3),
                self.scale.x * (c1 * s2 * s3 - c3 * s1),
                0.0),
            y : Vector4::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3),
                self.scale.y * (c2 * c3),
                self.scale.y * (c1 * c3 * s2 + s1 * s3),
                0.0),
            z: Vector4::new(
                self.scale.z * (c2 * s1),
                self.scale.z * (-s2),
                self.scale.z * (c1 * c2),
                0.0),
            w: Vector4::new(self.translation.x, self.translation.y, self.translation.z, 1.0)
        }
    }
}
impl Default for TransformComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vector3::<f32> = Vector3::<f32>::new(0.0, 0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector3::<f32> = Vector3::<f32>::new(1.0, 1.0, 1.0);
        // No rotation for default
        let rotation: Vector3::<Rad<f32>> = Vector3::<Rad<f32>>::new(Rad(0.0), Rad(0.0), Rad(0.0));
        Self {
            translation,
            scale,
            rotation,
        }
    }
}

#[derive(Debug)]
pub struct Transform2dComponent {
    pub translation: Vector2::<f32>,
    pub scale: Vector2::<f32>,
    pub rotation: f32,
}

impl Transform2dComponent {
    pub fn new(translation: Vector2::<f32>, scale: Vector2::<f32>, rotation: f32) -> Self {
        Self {
            translation,
            scale,
            rotation
        }
    }

    pub fn mat2(&self) -> Matrix2::<f32> {
        let sin: f32 = self.rotation.sin();
        let cos: f32 = self.rotation.cos();
        let rot_mat = Matrix2::<f32>::new(cos, sin, -sin, cos);
        let scale_mat = Matrix2::<f32>::new(self.scale.x, 0.0, 0.0, self.scale.y);
        rot_mat * scale_mat
    }
}
impl Default for Transform2dComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vector2::<f32> = Vector2::<f32>::new(0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector2::<f32> = Vector2::<f32>::new(1.0, 1.0);
        // No rotation for default
        let rotation: f32 = 0.0;
        Self {
            translation,
            scale,
            rotation,
        }
    }
}

#[derive(Debug)]
pub struct ModelComponent {
    pub verticies: Vec<Vertex>,
}

impl ModelComponent {
    pub fn new(verticies: Vec<Vertex>) -> Self {
        Self {
            verticies,
        }
    }
}


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