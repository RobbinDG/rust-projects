use std::fmt;
use std::marker::PhantomData;

use serde::{de, Deserialize, Deserializer};
use serde::de::{Error, SeqAccess, Visitor};

use crate::cube::Cube;
use crate::hit::Hit;
use crate::object::Object;
use crate::ray::Ray;
use crate::sphere::Sphere;
use crate::vector::Vector;

enum AllObjects {
    Sphere(Sphere),
    Cube(Cube),
}

impl Object for AllObjects {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        match self {
            AllObjects::Sphere(s) => s.intersect(ray),
            AllObjects::Cube(c) => c.intersect(ray),
        }
    }
}

impl<'de> Deserialize<'de> for AllObjects {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        if let Ok(sphere) = serde_json::from_value::<Sphere>(value.clone()) {
            return Ok(AllObjects::Sphere(sphere));
        }

        if let Ok(cube) = serde_json::from_value::<Cube>(value.clone()) {
            return Ok(AllObjects::Cube(cube));
        }

        Err(de::Error::custom("Object type unknown"))
    }
}

impl<'de> Deserialize<'de> for Box<dyn Object> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let object_type = AllObjects::deserialize(deserializer)?;
        Ok(Box::new(object_type))
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for Vector<T, N>
where
    T: Deserialize<'de> + Default + Copy,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayVisitor<T, const N: usize> {
            marker: PhantomData<T>,
        }

        impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
        where
            T: Deserialize<'de> + Default + Copy,
        {
            type Value = [T; N];

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(&format!("an array of length {}", N))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<[T; N], A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = [T::default(); N];
                for i in 0..N {
                    arr[i] = seq
                        .next_element()?
                        .ok_or_else(|| Error::invalid_length(i, &self))?;
                }
                Ok(arr)
            }
        }

        let vals = deserializer.deserialize_tuple(N, ArrayVisitor::<T, N> {
            marker: PhantomData,
        })?;
        Ok(Vector::new(vals))
    }
}