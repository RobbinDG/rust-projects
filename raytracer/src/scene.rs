use crate::sphere::Sphere;
use crate::vector::Vector;

pub struct Camera {
    pub eye_pos: Vector<f64, 3>,  // E vector
    pub fov: f64,
    pub width: u32,
    pub height: u32,
}
pub struct Scene {
    pub camera: Camera,
    pub objects: Vec<Sphere>,
}