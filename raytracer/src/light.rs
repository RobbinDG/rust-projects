use serde::Deserialize;

use crate::colour::Colour;
use crate::vector::Vector;

pub trait Light {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3>;

    fn colour(&self) -> Colour;
}

#[derive(Deserialize)]
pub struct PointLight {
    centre: Vector<f64, 3>,
}

impl Light for PointLight {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3> {
        (&self.centre - point).normalise()
    }

    fn colour(&self) -> Colour {
        Colour::new_rgba([255, 255, 255, 255])
    }
}