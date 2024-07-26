use crate::colour::Colour;
use crate::ray::Ray;
use crate::vector::Vector;

pub trait Object {
    fn intersect(&self, ray: &Ray) -> Option<(Vector<f64, 3>, f64)>;

    fn normal(&self, at: &Vector<f64, 3>) -> Vector<f64, 3>;

    fn material(&self) -> Colour;
}