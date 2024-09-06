use serde::Deserialize;
use crate::composition::{Colour, Light};
use crate::vector::Vector;

#[derive(Deserialize)]
pub struct PointLight {
    centre: Vector<f64, 3>,
    colour: Colour,
}

impl Light for PointLight {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3> {
        (&self.centre - point).normalise()
    }

    fn colour(&self) -> Colour {
        Colour::new_rgba([255, 255, 255, 255])
    }
}