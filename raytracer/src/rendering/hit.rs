use crate::composition::{Colour, Material};
use crate::vector::Vector;

pub struct Hit {
    pub loc: Vector<f64, 3>,
    pub t: f64,
    pub normal: Vector<f64, 3>,
    pub material: Material,
    pub back_side: bool,
    pub uv: Option<(f64, f64)>,
}

impl Hit {
    pub fn colour_at(&self) -> Colour {
        if let Some((u, v)) = self.uv {
            return self.material.colour_at(u, v);
        }
        self.material.colour.clone()
    }
}