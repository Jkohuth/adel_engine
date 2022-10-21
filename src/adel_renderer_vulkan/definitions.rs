use nalgebra::{ Matrix2, Matrix4,
    Rotation3,
    Translation3,
    Vector2, Vector3, Vector4,};
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
        // Since I am transitioning to Nalgebra should I just store the values as their types?
        let translation: Translation3<f32> = Translation3::from(self.translation);
        let rotation

        let mut transform =     Matrix4::<f32>::from_translation(self.translation);
        transform = transform * Matrix4::<f32>::from_rotation_y(self.rotation.y);
        transform = transform * Matrix4::<f32>::from_rotation_x(self.rotation.x);
        transform = transform * Matrix4::<f32>::from_rotation_z(self.rotation.z);
        // TODO: Right now scale only works as a scalar (shocking) do non-scalar scaling
        transform = transform * Matrix4::from_scale(self.scale);
        transform
    }
    pub fn mat4_less_computation(&self) -> Matrix4<f32> {
        let c3 = self.rotation.z.cos();
        let s3 = self.rotation.z.sin();
        let c2 = self.rotation.x.cos();
        let s2 = self.rotation.x.sin();
        let c1 = self.rotation.y.cos();
        let s1 = self.rotation.y.sin();
        Matrix4::<f32> {
            x_axis: Vector4::<f32>::new(
                self.scale.x * (c1 * c3 + s1 * s2 * s3), // 00
                self.scale.x * (c2 * s3),                // 01
                self.scale.x * (c1 * s2 * s3 - c3 * s1), // 02
                0.0),                                    // 03
            y_axis: Vector4::<f32>::new(
                self.scale.y * (c3 * s1 * s2 - c1 * s3), // 10
                self.scale.y * (c2 * c3),                // 11
                self.scale.y * (c1 * c3 * s2 + s1 * s3), // 12
                0.0),                                    // 13
            z_axis: Vector4::<f32>::new(
                self.scale.z * (c2 * s1),                // 20
                self.scale.z * (-s2),                    // 21
                self.scale.z * (c1 * c2),                // 22
                0.0),                                    // 23
            w_axis: Vector4::<f32>::new(
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
        let translation: Vector3<f32> = Vector3<f32>::new(0.0, 0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector3<f32> = Vector3<f32>::new(1.0, 1.0, 1.0);
        // No rotation for default
        let rotation: Vector3<f32> = Vector3<f32>::new(0.0, 0.0, 0.0);
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
        // This may be wrong, I haven't used 2D rendering with glam
        let rot_mat = Matrix2<f32>::from_cols(
            Vector2<f32>::new(cos, -sin), // 00 10
            Vector2<f32>::new(sin, cos)  // 10 11
        );
        let scale_mat = Matrix2<f32>::from_cols(
            Vector2<f32>::new(self.scale.x, 0.0),
            Vector2<f32>::new(0.0, self.scale.y)
        );
        /* {
            x_axis: Vector2<f32>::new(0.0, 0.0),
            y_axis: Vector2<f32>::new(0.0, 0.0),
        }
        let rot_mat = Matrix2::<f32>::new(cos, sin, -sin, cos);
        let scale_mat = Matrix2::<f32>::new(self.scale.x, 0.0, 0.0, self.scale.y);*/
        rot_mat * scale_mat
    }
}
impl Default for Transform2dComponent {
    fn default() -> Self {
        // No translation for default
        let translation: Vector2<f32> = Vector2<f32>::new(0.0, 0.0);
        // Default Scale needs to be 1
        let scale: Vector2<f32> = Vector2<f32>::new(1.0, 1.0);
        // No rotation for default
        let rotation: f32 = 0.0;
        Self {
            translation,
            scale,
            rotation,
        }
    }
}
