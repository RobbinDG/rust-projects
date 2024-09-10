use serde::{de, Deserialize, Deserializer};
use serde::de::Error;
use crate::composition::Object;
use crate::composition::objects::cube::Cube;
use crate::composition::objects::sphere::Sphere;
use crate::composition::objects::square::Square;
use crate::rendering::hit::Hit;
use crate::rendering::ray::Ray;

pub enum AllObjects {
    Sphere(Sphere),
    Cube(Cube),
    Square(Square),
}

impl Object for AllObjects {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        match self {
            AllObjects::Sphere(s) => s.intersect(ray),
            AllObjects::Cube(c) => c.intersect(ray),
            AllObjects::Square(s) => s.intersect(ray),
        }
    }
}

impl<'de> Deserialize<'de> for AllObjects {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Ok(sphere) = serde_json::from_value::<Sphere>(value.clone()) {
            return Ok(AllObjects::Sphere(sphere));
        }

        if let Ok(cube) = serde_json::from_value::<Cube>(value.clone()) {
            return Ok(AllObjects::Cube(cube));
        }

        if let Ok(square) = serde_json::from_value::<Square>(value.clone()) {
            return Ok(AllObjects::Square(square));
        }

        Err(de::Error::custom("Object type unknown"))
    }
}