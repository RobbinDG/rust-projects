use std::fmt;
use std::marker::PhantomData;

use serde::{Deserialize, Deserializer};
use serde::de::{Error, SeqAccess, Visitor};
use crate::composition::lights::all::AllLights;
use crate::composition::lights::light::Light;
use crate::composition::objects::all::AllObjects;
use crate::composition::objects::object::Object;
use crate::vector::Vector;

impl<'de> Deserialize<'de> for Box<dyn Object> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let object_type = AllObjects::deserialize(deserializer)?;
        Ok(Box::new(object_type))
    }
}

impl<'de> Deserialize<'de> for Box<dyn Light> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let light_type = AllLights::deserialize(deserializer)?;
        Ok(Box::new(light_type))
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