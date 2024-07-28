use crate::colour::Colour;
use crate::hit::Hit;
use crate::ray::Ray;
use crate::vector::Vector;

pub trait Object {
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}