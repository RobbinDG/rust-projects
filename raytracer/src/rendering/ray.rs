use crate::composition::{Colour, Material};
use crate::vector::Vector;

pub struct Ray {
    pub s: Vector<f64, 3>,
    pub d: Vector<f64, 3>,
    pub in_material: Material,
}

impl Ray {
    pub fn new(s: Vector<f64, 3>, d: Vector<f64, 3>) -> Ray {
        Ray {
            s,
            d: d.normalise(),
            in_material: Material::air(),
        }
    }

    pub fn at(&self, t: f64) -> Vector<f64, 3> {
        self.s + &self.d * t
    }
}