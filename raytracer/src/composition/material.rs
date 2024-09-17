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
}