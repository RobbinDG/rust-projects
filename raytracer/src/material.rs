use serde::Deserialize;
use crate::colour::Colour;

#[derive(Clone, Debug, Deserialize)]
pub struct Material {
    pub colour: Colour,
    pub ka: f64,
    pub kd: f64,
    pub ks: f64,
    pub alpha: f64
}