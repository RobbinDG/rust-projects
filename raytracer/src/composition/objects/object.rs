use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;
use as_any::AsAny;

pub trait Object: AsAny {
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}