use crate::hit::Hit;
use crate::ray::Ray;

pub trait Object {
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}