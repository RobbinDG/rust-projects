use crate::composition::Colour;
use crate::vector::Vector;

pub struct Ray {
    pub s: Vector<f64, 3>,
    pub d: Vector<f64, 3>,
    pub c: Colour,
}

impl Ray {
    pub fn new(s: Vector<f64, 3>, d: Vector<f64, 3>) -> Ray {
        Ray {
            s,
            d: d.normalise(),
            c: Colour::new_rgba([255, 255, 255, 255]),
        }
    }

    pub fn at(&self, t: f64) -> Vector<f64, 3> {
        self.s + &self.d * t
    }
}