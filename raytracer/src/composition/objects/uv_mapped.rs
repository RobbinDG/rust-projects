use crate::rendering::hit::Hit;

pub trait UVMapped {
    fn get_uv_coords(&self, hit: &Hit) -> (f64, f64);
}