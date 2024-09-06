use serde::Deserialize;
use crate::composition::camera::Camera;
use crate::composition::Colour;
use crate::composition::lights::light::Light;
use crate::composition::objects::object::Object;
#[derive(Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub background: Colour,
    pub objects: Vec<Box<dyn Object>>,
    pub lights: Vec<Box< dyn Light>>,
}