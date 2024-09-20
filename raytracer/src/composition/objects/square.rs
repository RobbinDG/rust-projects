use serde::Deserialize;
use crate::composition::{Material, Object};
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct Square {
    normal: Vector<f64, 3>,
    pos: Vector<f64, 3>,
    size: f64,
    rotation_deg: f64,
    material: Material,
}

impl Object for Square {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // d . normal >! 0
        // n . (s + dt - pos) = 0
        // n . (s - pos) + t (n . d) = 0
        // t (n . d) = - n . (s - pos)
        // t = -n . (s - pos) / (n . d)
        let nd = ray.d.dot(&self.normal);
        if nd == 0.0 {
            return None;
        }
        let t = -self.normal.dot(&(&ray.s - &self.pos)) / nd;
        Some(Hit {
            loc: ray.at(t),
            t,
            normal: self.normal,
            material: self.material.clone(),
            back_side: nd > 0.0,
        })
    }
}