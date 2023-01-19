use nalgebra::{Matrix4, Vector3};
use more_asserts;

pub struct Camera {
    //pub position: Vector3::<f32>,
    projection_matrix: Matrix4<f32>,
    view_matrix: Matrix4<f32>,
    _name: &'static str,
}
impl Camera {
    pub fn new() -> Self {
        Self {
            //position: Vector3::<f32>::default(),
            projection_matrix: Matrix4::identity(),
            view_matrix: Matrix4::identity(),
            _name: "camera",
        }
    }
    // TODO: Need to be written with a Vector3 supplied for position of the camera
    pub fn set_orthographic_projection(&mut self, left: f32, right: f32, top: f32, bottom: f32, near: f32, far: f32) {
        self.projection_matrix = Matrix4::identity();

        self.projection_matrix[(0, 0)] = 2.0 / (right - left);
        self.projection_matrix[(1, 1)] = 2.0 / (bottom - top);
        self.projection_matrix[(2, 2)] = 1.0 / (far - near);
        self.projection_matrix[(3, 0)] = -(right + left) / (right - left);
        self.projection_matrix[(3, 1)] = -(bottom + top) / (bottom - top);
        self.projection_matrix[(3, 2)] = -near / (far - near);

    }
    /*pub fn set_orthographic_projection_pos(&mut self, half_extent_h: f32, half_extent_v: f32, depth: f32) {
        self.projection_matrix = Matrix4::identity();
        let right =  self.position.x + half_extent_h;
        let left =   self.position.x - half_extent_h;
        let bottom = self.position.y + half_extent_v;
        let top =    self.position.y - half_extent_v;
        let near =   self.position.z;
        let far =    self.position.z + depth;

        self.projection_matrix[(0, 0)] = 2.0 / (right - left);
        self.projection_matrix[(1, 1)] = 2.0 / (bottom - top);
        self.projection_matrix[(2, 2)] = 1.0 / (far - near);
        self.projection_matrix[(3, 0)] = -(right + left) / (right - left);
        self.projection_matrix[(3, 1)] = -(bottom + top) / (bottom - top);
        self.projection_matrix[(3, 2)] = -near / (far - near);

    }*/
    pub fn set_perspective_projection(&mut self, fovy: f32, aspect: f32, near: f32, far: f32) {
        more_asserts::assert_gt!((aspect - f32::MIN).abs(), 0.0);

        let tan_half_fovy = f32::tan(fovy/2.0);
        self.projection_matrix = Matrix4::identity();

        //self.projection_matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        //self.projection_matrix[(1, 1)] = 1.0 / (tan_half_fovy);
        //self.projection_matrix[(2, 2)] = far / (far - near);
        //self.projection_matrix[(2, 3)] = 1.0;
        //self.projection_matrix[(3, 2)] = -(far * near) / (far - near);

        self.projection_matrix[(0, 0)] = 1.0 / (aspect * tan_half_fovy);
        self.projection_matrix[(1, 1)] = 1.0 / (tan_half_fovy);
        self.projection_matrix[(2, 2)] = far / (far - near);
        self.projection_matrix[(3, 2)] = 1.0;
        self.projection_matrix[(2, 3)] = -(far * near) / (far - near);
    }

    pub fn set_view_direction(&mut self, position: Vector3<f32>, direction: Vector3<f32>, up: Option<Vector3<f32>>) {
        let w = Vector3::normalize(&direction);
        //let w = direction.normalize();
        // Supply default up value if None was passed into the function
        let u = Vector3::normalize(&w.cross(&up.unwrap_or(Vector3::new(0.0, -1.0, 0.0))));
        let v = w.cross(&u);

        self.view_matrix = Matrix4::identity();
        self.view_matrix[(0, 0)] = u.x;
        self.view_matrix[(1, 0)] = u.y;
        self.view_matrix[(2, 0)] = u.z;
        self.view_matrix[(0, 1)] = v.x;
        self.view_matrix[(1, 1)] = v.y;
        self.view_matrix[(2, 1)] = v.z;
        self.view_matrix[(0, 2)] = w.x;
        self.view_matrix[(1, 2)] = w.y;
        self.view_matrix[(2, 2)] = w.z;
        self.view_matrix[(3, 0)] = -Vector3::dot(&u, &position);
        self.view_matrix[(3, 1)] = -Vector3::dot(&v, &position);
        self.view_matrix[(3, 2)] = -Vector3::dot(&w, &position);

    }

    pub fn set_view_target(&mut self, position: Vector3::<f32>, target: Vector3::<f32>, up: Option<Vector3::<f32>>) {
        more_asserts::assert_gt!(target.dot(&target), f32::MIN);
        self.set_view_direction(position, target - position, up);
    }

    pub fn set_view_yxz(&mut self, position: Vector3::<f32>, rotation: Vector3::<f32>) {
      let c3 = f32::cos(rotation.z);
      let s3 = f32::sin(rotation.z);
      let c2 = f32::cos(rotation.x);
      let s2 = f32::sin(rotation.x);
      let c1 = f32::cos(rotation.y);
      let s1 = f32::sin(rotation.y);
      let u = Vector3::new(c1 * c3 + s1 * s2 * s3, c2 * s3, c1 * s2 * s3 - c3 * s1);
      let v = Vector3::new(c3 * s1 * s2 - c1 * s3, c2 * c3, c1 * c3 * s2 + s1 * s3);
      let w = Vector3::new(c2 * s1, -s2, c1 * c2);
      //self.view_matrix = Matrix4::identity();
      //self.view_matrix[(0, 0)] = u.x;
      //self.view_matrix[(1, 0)] = u.y;
      //self.view_matrix[(2, 0)] = u.z;
      //self.view_matrix[(0, 1)] = v.x;
      //self.view_matrix[(1, 1)] = v.y;
      //self.view_matrix[(2, 1)] = v.z;
      //self.view_matrix[(0, 2)] = w.x;
      //self.view_matrix[(1, 2)] = w.y;
      //self.view_matrix[(2, 2)] = w.z;
      //self.view_matrix[(3, 0)] = -Vector3::dot(&u, &position);
      //self.view_matrix[(3, 1)] = -Vector3::dot(&v, &position);
      //self.view_matrix[(3, 2)] = -Vector3::dot(&w, &position);

      // Nalgebra seems to be organize weirdly
      self.view_matrix[(0, 0)] = u.x;
      self.view_matrix[(0, 1)] = u.y;
      self.view_matrix[(0, 2)] = u.z;
      self.view_matrix[(1, 0)] = v.x;
      self.view_matrix[(1, 1)] = v.y;
      self.view_matrix[(1, 2)] = v.z;
      self.view_matrix[(2, 0)] = w.x;
      self.view_matrix[(2, 1)] = w.y;
      self.view_matrix[(2, 2)] = w.z;
      self.view_matrix[(0, 3)] = -Vector3::dot(&u, &position);
      self.view_matrix[(1, 3)] = -Vector3::dot(&v, &position);
      self.view_matrix[(2, 3)] = -Vector3::dot(&w, &position);
    }
    pub fn get_projection(&self) -> Matrix4::<f32> {
        self.projection_matrix
    }
    pub fn get_view(&self) -> Matrix4::<f32> {
        self.view_matrix
    }
}
