use crate::ray::Ray;
use crate::vector::Vector;

struct Square {
    c: Vector<f64, 3>,
    r: f64,
}

impl Square {
    pub fn intersect(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64)>  {


        None
    }
}