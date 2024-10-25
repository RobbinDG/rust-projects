use std::sync::Arc;
use crate::composition::colour::Colour;
use image::{DynamicImage, GenericImageView, ImageReader};
use once_cell::sync::Lazy;
use serde::Deserialize;

static EARTH: Lazy<Arc<DynamicImage>> = Lazy::new(
    || {
        let img =ImageReader::open("resources/earth.jpg").unwrap().decode().unwrap();
        Arc::new(img)
    }
);
#[derive(Clone, Debug, Deserialize)]
pub struct Material {
    pub colour: Colour,
    pub ka: f64,
    pub kd: f64,
    pub ks: f64,
    pub alpha: f64,
    pub reflectivity: Option<f64>,
    pub transmittance: Option<f64>,
    pub refractive_index: Option<f64>,
    pub texture: Option<String>,
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
            refractive_index: Some(1.0),
            texture: None,
        }
    }

    pub fn colour_at(&self, u: f64, v: f64) -> Colour {
        match &self.texture {
            None => self.colour.clone(),
            Some(texture) => {
                let img = Arc::clone(&EARTH);
                let x = (img.width() as f64 * u) as u32;
                let y = (img.height() as f64 * v) as u32;
                Colour::from_pixel(img.get_pixel(x, y))
            }
        }
    }
}