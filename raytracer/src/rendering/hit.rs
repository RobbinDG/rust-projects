use crate::composition::Material;
use crate::vector::Vector;

pub struct Hit {
    pub loc: Vector<f64, 3>,
    pub t: f64,
    pub normal: Vector<f64, 3>,
    pub material: Material,
    pub back_side: bool,
}