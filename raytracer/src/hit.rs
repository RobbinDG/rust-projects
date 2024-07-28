use crate::colour::Colour;
use crate::vector::Vector;

pub struct Hit {
    pub loc: Vector<f64, 3>,
    pub t: f64,
    pub normal: Vector<f64, 3>,
    pub material: Colour,
}