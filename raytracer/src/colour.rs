use std::ops;

#[derive(Copy, Clone)]
pub struct Colour {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl ops::Mul<f64> for &Colour {
    type Output = Colour;

    fn mul(self, rhs: f64) -> Self::Output {
        Colour {
            r: (self.r as f64 * rhs) as u8,
            g: (self.g as f64 * rhs) as u8,
            b: (self.b as f64 * rhs) as u8,
            a: self.a,
        }
    }
}