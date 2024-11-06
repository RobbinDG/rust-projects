use std::str;
use std::str::{FromStr, Utf8Error};

#[derive(Debug)]
pub struct ServerResponse {
    pub payload: String,
}

impl ServerResponse {
    pub fn from_str(message: &str) -> Self {
        ServerResponse { payload: String::from_str(message).unwrap() }
    }

    pub fn parse(response: &[u8]) -> Result<Self, Utf8Error> {
        let header = "res:";
        let resp_str = str::from_utf8(response)?;
        Ok(Self {
            payload: resp_str[header.len()..].trim_matches('\0').to_string(),
        })
    }

    pub fn as_payload(&self) -> String {
        let header = "res:";
        header.to_string() + &self.payload
    }
}