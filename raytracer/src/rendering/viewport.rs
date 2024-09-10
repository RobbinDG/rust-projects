use crate::composition::Camera;
use crate::rendering::ray::Ray;
use crate::vector::Vector;
use std::f64::consts::PI;

pub struct Viewport {
    eye: Vector<f64, 3>,
    qx: Vector<f64, 3>,
    qy: Vector<f64, 3>,
    p11: Vector<f64, 3>,
}

impl Viewport {
    pub fn new(camera: &Camera) -> Viewport {
        let t = camera.dir;
        let v = Vector::new([0.0, 1.0, 0.0]); // Up-vector
        let b = &v.cross(&t); // Side-vector

        let tn = t.normalise();
        let bn = b.normalise();
        let vn = v.normalise();

        let fov = camera.fov / 180.0 * PI;
        let gx = (fov / 2.0).tan();
        let gy = gx * (camera.height - 1) as f64 / (camera.width - 1) as f64;
        let qx = &bn * ((2.0 * gx) / (camera.width - 1) as f64);
        let qy = &vn * ((2.0 * gy) / (camera.height - 1) as f64);

        let p11 = &(&tn - &(&bn * gx)) + &(&vn * gy);
        Viewport {
            eye: camera.eye_pos.clone(),
            qx,
            qy,
            p11,
        }
    }

    pub fn ray_for_pixel(&self, x: usize, y: usize) -> Ray {
        let pij = &(self.p11 + &(&self.qx * x as f64)) - &(&self.qy * y as f64);
        Ray::new(self.eye , pij.normalise())
    }
}
