use std::ops::{Add, Mul};

use serde::Deserialize;

use crate::vector::Vector;

#[derive(Debug, Deserialize, Clone)]
pub struct Colour {
    rgba: Vector<f64, 4>,
}

impl Colour {
    pub fn new_rgba(rgba: [u8; 4]) -> Colour {
        Colour {
            rgba: Vector::new([
                rgba[0] as f64 / 255.0,
                rgba[1] as f64 / 255.0,
                rgba[2] as f64 / 255.0,
                rgba[3] as f64 / 255.0,
            ])
        }
    }

    pub fn r(&self) -> u8 { (self.rgba[0] * 255.0) as u8 }
    pub fn g(&self) -> u8 { (self.rgba[1] * 255.0) as u8 }
    pub fn b(&self) -> u8 { (self.rgba[2] * 255.0) as u8 }
    pub fn a(&self) -> u8 { (self.rgba[3] * 255.0) as u8 }

    pub fn black() -> Colour {
        Self::new_rgba([0, 0, 0, 0])
    }
}

impl Add<Colour> for Colour {
    type Output = Colour;

    fn add(self, rhs: Colour) -> Self::Output {
        Colour { rgba: &self.rgba + &rhs.rgba }
    }
}

impl Mul<f64> for &Colour {
    type Output = Colour;

    fn mul(self, rhs: f64) -> Self::Output {
        Colour { rgba: &self.rgba * rhs }
    }
}

impl Mul<&Colour> for &Colour {
    type Output = Colour;

    fn mul(self, rhs: &Colour) -> Self::Output {
        Colour { rgba: &self.rgba * &rhs.rgba }
    }
}