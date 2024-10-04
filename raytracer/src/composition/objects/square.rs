use serde::Deserialize;
use crate::composition::{Material, Object};
use crate::composition::objects::plane::Plane;
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct Square {
    plane: Plane,
    size: f64,
    rotation_deg: f64,
}

impl Object for Square {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let plane_hit = self.plane.intersect(ray)?;
        // let (par_1, par_2) = self.plane.parallels();
        let dist_vec = &plane_hit.loc - &self.plane.pos;
        if dist_vec.abs().max() > self.size {
            return None
        }
        Some(plane_hit)
    }
}