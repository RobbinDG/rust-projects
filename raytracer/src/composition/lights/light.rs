use serde::Deserialize;

use crate::composition::colour::Colour;
use crate::vector::Vector;

pub trait Light {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3>;

    fn colour(&self) -> Colour;
}

#[derive(Deserialize)]
pub struct Sun {
    ray_dir: Vector<f64, 3>,
    colour: Colour,
}

impl Light for Sun {
    fn vec(&self, _: &Vector<f64, 3>) -> Vector<f64, 3> {
        -&self.ray_dir // TODO this is inefficient and can be precalculated
    }

    fn colour(&self) -> Colour {
        self.colour.clone()
    }
}