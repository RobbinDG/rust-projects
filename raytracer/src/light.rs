use serde::Deserialize;
use crate::vector::Vector;

pub trait Light {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3>;
}

#[derive(Deserialize)]
pub struct PointLight {
    centre: Vector<f64, 3>,
}

impl Light for PointLight {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3> {
        return &self.centre - point;
    }
}