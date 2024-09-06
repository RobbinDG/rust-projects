use serde::{Deserialize, Deserializer};
use serde::de::Error;
use crate::composition::{Colour, Light};
use crate::composition::lights::light::Sun;
use crate::composition::lights::point::PointLight;
use crate::vector::Vector;

pub enum AllLights {
    PointLight(PointLight),
    Sun(Sun),
}

impl Light for AllLights {
    fn vec(&self, point: &Vector<f64, 3>) -> Vector<f64, 3> {
        match self {
            AllLights::PointLight(p) => p.vec(point),
            AllLights::Sun(s) => s.vec(point),
        }
    }

    fn colour(&self) -> Colour {
        match self {
            AllLights::PointLight(p) => p.colour(),
            AllLights::Sun(s) => s.colour(),
        }
    }
}

impl<'de> Deserialize<'de> for AllLights {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Ok(point_light) = serde_json::from_value::<PointLight>(value.clone()) {
            return Ok(AllLights::PointLight(point_light));
        }
        if let Ok(sun) = serde_json::from_value::<Sun>(value.clone()) {
            return Ok(AllLights::Sun(sun));
        }

        Err(Error::custom("Light type unknown"))
    }
}