use ash::vk;
use std::hash::{Hash, Hasher};

use nalgebra::{Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};

#[derive(Default)]
pub struct VertexBuilder {
    position: Vector3<f32>,
    color: Vector3<f32>,
    normal: Vector3<f32>,
    uv: Vector2<f32>,
}

impl VertexBuilder {
    pub fn new() -> Self {
        VertexBuilder {
            position: Vector3::<f32>::default(),
            color: Vector3::<f32>::default(),
            normal: Vector3::<f32>::default(),
            uv: Vector2::<f32>::default(),
        }
    }
    pub fn build(&self) -> Vertex {
        Vertex {
            position: self.position,
            color: self.color,
            normal: self.normal,
            uv: self.uv,
        }
    }
    pub fn position(mut self, position: Vector3<f32>) -> Self {
        self.position = position;
        self
    }
    pub fn color(mut self, color: Vector3<f32>) -> Self {
        self.color = color;
        self
    }
    pub fn normal(mut self, normal: Vector3<f32>) -> Self {
        self.normal = normal;
        self
    }
    pub fn uv(mut self, uv: Vector2<f32>) -> Self {
        self.uv = uv;
        self
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub uv: Vector2<f32>,
}

impl Vertex {
    pub fn builder() -> VertexBuilder {
        VertexBuilder::default()
    }
    pub fn binding_descriptions() -> [vk::VertexInputBindingDescription; 1] {
        [vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()]
    }
    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 4] {
        let pos_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();
        let color_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(std::mem::size_of::<nalgebra::Vector3<f32>>() as u32)
            .build();
        let normal_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(
                (std::mem::size_of::<nalgebra::Vector3<f32>>()
                    + std::mem::size_of::<nalgebra::Vector3<f32>>()) as u32,
            )
            .build();
        let uv_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(3)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(
                (std::mem::size_of::<nalgebra::Vector3<f32>>()
                    + std::mem::size_of::<nalgebra::Vector3<f32>>()
                    + std::mem::size_of::<nalgebra::Vector3<f32>>()) as u32,
            )
            .build();
        [pos_attrib, color_attrib, normal_attrib, uv_attrib]
    }
}
impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position
            && self.color == other.color
            && self.normal == other.normal
            && self.uv == other.uv
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.normal[0].to_bits().hash(state);
        self.normal[1].to_bits().hash(state);
        self.normal[2].to_bits().hash(state);
        self.uv[0].to_bits().hash(state);
        self.uv[1].to_bits().hash(state);
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct UniformBufferObject {
    pub projection_view: nalgebra::Matrix4<f32>,
    pub ambient_light_color: nalgebra::Vector4<f32>,
    // This should be vector3 but alignment is a problem
    pub light_position: nalgebra::Vector4<f32>,
    pub light_color: nalgebra::Vector4<f32>,
}
#[repr(C)]
#[derive(Debug)]
pub struct PushConstantData {
    pub model_matrix: nalgebra::Matrix4<f32>,
    pub normal_matrix: nalgebra::Matrix4<f32>,
}

#[derive(Debug, Copy, Clone)]
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
        Matrix4::from_columns(&[
            Vector4::<f32>::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                self.scale.x * (c2 * s3),                // 10
                self.scale.x * (c1 * s2 * s3 - c3 * s1), // 20
                0.0,                                     // 30
            ),
            Vector4::<f32>::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3), // 01
                self.scale.y * (c2 * c3),                // 11
                self.scale.y * (c1 * c3 * s2 + s1 * s3), // 21
                0.0,                                     // 31
            ),
            Vector4::<f32>::new(
                self.scale.z * (c2 * s1), // 02
                self.scale.z * (-s2),     // 12
                self.scale.z * (c1 * c2), // 22
                0.0,                      // 32
            ),
            Vector4::<f32>::new(
                self.translation.x, // 03
                self.translation.y, // 13
                self.translation.z, // 23
                1.0,                // 33
            ),
        ])
    }
    pub fn normal_matrix_mat3(&self) -> Matrix3<f32> {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        let inverse_scale =
            Vector3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        Matrix3::from_columns(&[
            Vector3::<f32>::new(
                inverse_scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                inverse_scale.x * (c2 * s3),                // 10
                inverse_scale.x * (c1 * s2 * s3 - c3 * s1), // 20
            ),
            Vector3::<f32>::new(
                inverse_scale.y * (c3 * s1 * s2 - c1 * s3), // 01
                inverse_scale.y * (c2 * c3),                // 11
                inverse_scale.y * (c1 * c3 * s2 + s1 * s3), // 21
            ),
            Vector3::<f32>::new(
                inverse_scale.z * (c2 * s1), // 02
                inverse_scale.z * (-s2),     // 12
                inverse_scale.z * (c1 * c2), // 22
            ),
        ])
    }
    pub fn normal_matrix_mat4(&self) -> Matrix4<f32> {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        let inverse_scale =
            Vector3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        Matrix4::from_columns(&[
            Vector4::<f32>::new(
                inverse_scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                inverse_scale.x * (c2 * s3),                // 10
                inverse_scale.x * (c1 * s2 * s3 - c3 * s1), // 20
                0.0,
            ),
            Vector4::<f32>::new(
                inverse_scale.y * (c3 * s1 * s2 - c1 * s3), // 01
                inverse_scale.y * (c2 * c3),                // 11
                inverse_scale.y * (c1 * c3 * s2 + s1 * s3), // 21
                0.0,
            ),
            Vector4::<f32>::new(
                inverse_scale.z * (c2 * s1), // 02
                inverse_scale.z * (-s2),     // 12
                inverse_scale.z * (c1 * c2), // 22
                0.0,
            ),
            Vector4::<f32>::new(0.0, 0.0, 0.0, 1.0),
        ])
    }
}
pub fn create_push_constant_data(
    model_matrix: Matrix4<f32>,
    normal_matrix: Matrix4<f32>,
) -> PushConstantData {
    PushConstantData {
        model_matrix,
        normal_matrix,
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
            rotation,
        }
    }

    pub fn mat2(&self) -> Matrix2<f32> {
        let sin: f32 = self.rotation.sin();
        let cos: f32 = self.rotation.cos();
        // This may be wrong, I haven't used 2D rendering with nalgebra
        let rot_mat = Matrix2::from_columns(&[
            Vector2::new(cos, -sin), // 00 10
            Vector2::new(sin, cos),  // 10 11
        ]);
        let scale_mat = Matrix2::from_columns(&[
            Vector2::new(self.scale.x, 0.0),
            Vector2::new(0.0, self.scale.y),
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
            Vector3::new(0.0, 0.0, 1.0),  // 20 21 22
        ]);
        let scale_mat = Matrix3::from_columns(&[
            Vector3::new(self.scale.x, 0.0, 0.0),
            Vector3::new(0.0, self.scale.y, 0.0),
            Vector3::new(0.0, 0.0, 1.0),
        ]);
        let mut rot_scale_translate = rot_mat * scale_mat;

        rot_scale_translate[(2, 0)] = self.translation[0];
        rot_scale_translate[(2, 1)] = self.translation[1];
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
