use std::ops::Mul;
use serde::Deserialize;
use crate::vector::Vector;

#[derive(Deserialize, Clone)]
pub struct Colour {
    rgba: Vector<f64, 4>
}

impl Colour {
    pub fn new_rgba(rgba: [u8; 4]) -> Colour {
        Colour {
            rgba: Vector::new([
                rgba[0] as f64,
                rgba[1] as f64,
                rgba[2] as f64,
                rgba[3] as f64,
            ])
        }
    }

    pub fn r(&self) -> u8 { self.rgba[0] as u8 }
    pub fn g(&self) -> u8 { self.rgba[1] as u8 }
    pub fn b(&self) -> u8 { self.rgba[2] as u8 }
    pub fn a(&self) -> u8 { self.rgba[3] as u8 }
}

impl Mul<f64> for &Colour {
    type Output = Colour;

    fn mul(self, rhs: f64) -> Self::Output {
        Colour{rgba: &self.rgba * rhs}
    }
}
