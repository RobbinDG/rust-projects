use serde::de::DeserializeOwned;
use serde::Serialize;

#[derive(Debug)]
pub struct CodecError(pub postcard::Error);

impl From<postcard::Error> for CodecError {
    fn from(e: postcard::Error) -> Self {
        Self(e)
    }
}

pub fn encode<T>(payload: &T) -> Result<Vec<u8>, CodecError>
where
    T: Serialize + DeserializeOwned,
{
    Ok(postcard::to_allocvec(payload)?)
}

pub fn decode<T>(encoded: &Vec<u8>) -> Result<T, CodecError>
where
    T: Serialize + DeserializeOwned,
{
    Ok(postcard::from_bytes(encoded)?)
}
