use crate::vector::Vector;

pub struct Ray {
    pub s: Vector<f64, 3>,
    pub d: Vector<f64, 3>,
}

impl Ray {
    pub fn new(s: Vector<f64, 3>, d: Vector<f64, 3>) -> Ray {
        Ray {
            s,
            d: d.normalise(),
        }
    }

    pub fn at(&self, t: f64) -> Vector<f64, 3> {
        self.s + &self.d * t
    }
}