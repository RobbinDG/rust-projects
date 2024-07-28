use crate::object::Object;
use crate::ray::Ray;
use crate::vector::Vector;

pub struct Camera {
    pub eye_pos: Vector<f64, 3>,  // E vector
    pub dir: Vector<f64, 3>,
    pub fov: f64,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn ray_for_pixel(&self, x: u32, y: u32) -> Ray {
        let rel_x = (2.0 * x as f64 - self.width as f64) / (self.width as f64);
        let rel_y = (2.0 * y as f64 - self.height as f64) / (self.height as f64);
        let rel_z = 1.0;

        let base = Vector::new([0.0, 1.0]);  // x = 0, z = 1
        let angled = Vector::new([self.dir.x(), self.dir.z()]);
        let theta = &base.signed_angle_to(&angled);

        let dir = Vector::new([
            rel_x * theta.cos() + rel_z * -theta.sin(),
            rel_y,
            rel_x * theta.sin() + rel_z * theta.cos(),
        ]);

        return Ray::new(self.eye_pos.clone(), dir);
    }
}

pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Box<dyn Object>>,
}