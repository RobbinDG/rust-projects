use serde::Deserialize;
use crate::composition::{Material, Object};
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct Plane {
    normal: Vector<f64, 3>,
    pub pos: Vector<f64, 3>,
    material: Material,
}

impl Plane {
    pub fn parallels(&self) -> (Vector<f64, 3>, Vector<f64, 3>) {
        (
            Vector::new([0.0, -self.normal[2], self.normal[1]]).normalise(),
            Vector::new([-self.normal[2], 0.0, self.normal[0]]).normalise(),
        )
    }
}

impl Object for Plane {
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
            uv: None,
        })
    }
}