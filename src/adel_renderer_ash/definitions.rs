use nalgebra::{
    Matrix2,
    Matrix3,
    Matrix4,
    Rotation3,
    RowVector4,
    Scale3,
    Translation3,
    Vector2, Vector3, Vector4,};

use nalgebra;
#[derive(Debug)]
#[repr(C)]
pub struct Vertex2d {
    pub position: nalgebra::Vector2::<f32>,
    pub color: nalgebra::Vector3::<f32>,
}
pub struct TriangleComponent {
    pub verticies: Vec<Vertex2d>
}
impl TriangleComponent {
    pub fn new(verticies: Vec<Vertex2d>) -> Self {
        assert_eq!(verticies.len(), 3);
        Self {
            verticies
        }
    }
}
use ash::vk::{Buffer, DeviceMemory};
// Bad name I know but this will go away soon
pub struct VertexIndexComponent {
    pub vertices : Vec<Vertex2d>,
    pub indices : Vec<u16>
}
// TODO: Make a better struct to be passed around
pub struct BufferComponent {
    pub vertex_buffer: Buffer,
    pub vertex_buffer_memory: DeviceMemory,
    pub index_buffer: Buffer,
    pub index_buffer_memory: DeviceMemory,
    pub indices_count: u32,
}
// TODO: Create separate files for Vertex specific structs
// TODO: Come up with a better name for this
pub struct VertexBuffer {
    pub buffer: Buffer,
    pub memory: DeviceMemory,
}
#[repr(C)]
#[derive(Debug)]
pub struct PushConstantData {
    pub transform: nalgebra::Matrix4<f32>,
    pub color: nalgebra::Vector3<f32>
}
#[repr(C)]
#[derive(Debug)]
pub struct PushConstantData2D {
    pub transform: nalgebra::Matrix2<f32>,
    pub color: nalgebra::Vector3<f32>
}
#[derive(Debug)]
pub struct TransformComponent {
    pub translation: Vector3<f32>,
    pub scale: Vector3<f32>,
    pub rotation: Vector3<f32>,
}

impl TransformComponent {
    pub fn new(translation: Vector3<f32>, scale: Vector3<f32>, rotation: Vector3<f32>) -> Self {
        Self {
            translation,
            scale,
            rotation,
        }
    }

    // Matrix is translate * Ry * Rx * Rz * Sx * Sy * Sz
    pub fn mat4(&self) -> Matrix4<f32> {
        let mut transform = Matrix4::new_translation(&self.translation);
        transform = transform * Matrix4::from_axis_angle(&Vector3::y_axis(), self.rotation.y);
        transform = transform * Matrix4::from_axis_angle(&Vector3::x_axis(), self.rotation.x);
        transform = transform * Matrix4::from_axis_angle(&Vector3::z_axis(), self.rotation.z);
        transform = transform.append_nonuniform_scaling(&self.scale);
        transform
    }
    pub fn mat4_less_computation(&self) -> Matrix4<f32> {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        Matrix4::from_rows(&[
            RowVector4::<f32>::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                self.scale.x * (c2 * s3),                // 01
                self.scale.x * (c1 * s2 * s3 - c3 * s1), // 02
                0.0),                                    // 03
            RowVector4::<f32>::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3), // 10
                self.scale.y * (c2 * c3),                // 11
                self.scale.y * (c1 * c3 * s2 + s1 * s3), // 12
                0.0),                                    // 13
            RowVector4::<f32>::new(
                self.scale.z * (c2 * s1),                // 20
                self.scale.z * (-s2),                    // 21
                self.scale.z * (c1 * c2),                // 22
                0.0),                                    // 23
            RowVector4::<f32>::new(
                self.translation.x,                      // 30
                self.translation.y,                      // 31
                self.translation.z,                      // 32
                1.0)                                     // 33
            ])
    }
}
pub fn create_push_constant_data(camera_projection: Matrix4<f32>, transform: &TransformComponent) -> PushConstantData {
    PushConstantData {
        transform: (camera_projection * transform.mat4_less_computation()),
        color: Vector3::new(0.0, 0.0, 0.0),
    }
}

impl Default for TransformComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector3<f32> = Vector3::new(1.0, 1.0, 1.0);
        // No rotation for default
        let rotation: Vector3<f32> = Vector3::new(0.0, 0.0, 0.0);
        Self {
            translation,
            scale,
            rotation,
        }
    }
}

#[derive(Debug)]
pub struct Transform2dComponent {
    pub translation: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: f32,
}

impl Transform2dComponent {
    pub fn new(translation: Vector2<f32>, scale: Vector2<f32>, rotation: f32) -> Self {
        Self {
            translation,
            scale,
            rotation
        }
    }

    pub fn mat2(&self) -> Matrix2<f32> {
        let sin: f32 = self.rotation.sin();
        let cos: f32 = self.rotation.cos();
        // This may be wrong, I haven't used 2D rendering with nalgebra
        let rot_mat = Matrix2::from_columns(&[
            Vector2::new(cos, -sin), // 00 10
            Vector2::new(sin, cos)  // 10 11
        ]);
        let scale_mat = Matrix2::from_columns(&[
            Vector2::new(self.scale.x, 0.0),
            Vector2::new(0.0, self.scale.y)
        ]);
        /* {
            x_axis: Vector2<f32>::new(0.0, 0.0),
            y_axis: Vector2<f32>::new(0.0, 0.0),
        }
        let rot_mat = Matrix2::<f32>::new(cos, sin, -sin, cos);
        let scale_mat = Matrix2::<f32>::new(self.scale.x, 0.0, 0.0, self.scale.y);*/
        rot_mat * scale_mat
    }
    // Using Homogenous Coordinates in 2D, can be done with a standard transform and probably
    // should to include a Z buffer, keeping things like this to better understand
    pub fn mat3(&self) -> Matrix3<f32> {
        let sin: f32 = self.rotation.sin();
        let cos: f32 = self.rotation.cos();
        // This may be wrong, I haven't used 2D rendering with nalgebra
        let rot_mat = Matrix3::from_columns(&[
            Vector3::new(cos, -sin, 0.0), // 00 10 20
            Vector3::new(sin, cos, 0.0),  // 10 11 21
            Vector3::new(0.0, 0.0, 1.0) // 20 21 22
        ]);
        let scale_mat = Matrix3::from_columns(&[
            Vector3::new(self.scale.x, 0.0, 0.0),
            Vector3::new(0.0, self.scale.y, 0.0),
            Vector3::new(0.0, 0.0, 1.0)
        ]);
        let mut rot_scale_translate = rot_mat * scale_mat;

        rot_scale_translate[(2,0)] = self.translation[0];
        rot_scale_translate[(2,1)] = self.translation[1];
        //log::info!("Mat3: {:?}", rot_scale_translate);
        rot_scale_translate
    }
}
impl Default for Transform2dComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vector2<f32> = Vector2::new(0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector2<f32> = Vector2::new(1.0, 1.0);
        // No rotation for default
        let rotation: f32 = 0.0;
        Self {
            translation,
            scale,
            rotation,
        }
    }
}
