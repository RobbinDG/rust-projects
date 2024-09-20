use serde::Deserialize;
use crate::composition::colour::Colour;

#[derive(Clone, Debug, Deserialize)]
pub struct Material {
    pub colour: Colour,
    pub ka: f64,
    pub kd: f64,
    pub ks: f64,
    pub alpha: f64,
    pub reflectivity: Option<f64>,
    pub transmittance: Option<f64>,
    pub refractive_index: Option<f64>
}

impl Material {
    pub fn air() -> Material {
        Material {
            colour: Colour::black(),
            ka: 0.0,
            kd: 0.0,
            ks: 0.0,
            alpha: 0.0,
            reflectivity: None,
            transmittance: None,
            refractive_index: Some(1.0)
        }
    }
}