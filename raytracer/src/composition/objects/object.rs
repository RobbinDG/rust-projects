use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;

pub trait Object {
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}