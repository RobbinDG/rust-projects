use std::io::Error;
use std::str;
use std::str::Utf8Error;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RequestError {
    IO(Error),
    Parsing(Utf8Error),
    Internal(String),
}

impl From<Error> for RequestError {
    fn from(value: Error) -> Self {
        RequestError::IO(value)
    }
}

impl From<Utf8Error> for RequestError {
    fn from(value: Utf8Error) -> Self {
        RequestError::Parsing(value)
    }
}

impl From<String> for RequestError {
    fn from(value: String) -> Self {
        RequestError::Internal(value)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerRequest {
    ListQueues,
    CheckQueue(String),
    CreateQueue(String),
    PutMessage(String, String),
}

impl ServerRequest {
    pub fn as_string(&self) -> String {
        match self {
            Self::ListQueues => String::from("list_queues"),
            Self::CheckQueue(s) => String::from("check_queue:") + s,
            Self::CreateQueue(s) => String::from("create_queue:") + s,
            Self::PutMessage(q, m) => String::from("put:") + q + "&" + m,
        }
    }

    fn from_str(s: &str) -> Result<Self, String> {
        println!("{}", s);
        let parts: Vec<&str> = s.split(':').collect();
        let (c, args_str) = parts.split_at(1);
        let command = c[0];
        let args: Vec<&str> = if args_str.len() > 0 {
            args_str[0].split('&').collect()
        } else {
            Vec::default()
        };
        match command {
            "list_queues" => Ok(Self::ListQueues),
            "check_queue" => Ok(Self::CheckQueue(String::from(args[0]))),
            "create_queue" => Ok(Self::CreateQueue(String::from(args[0]))),
            "put" => Ok(Self::PutMessage(String::from(args[0]), String::from(args[1]))),
            _ => Err("Command not recognized".to_string()),
        }
    }

    pub fn as_payload(&self) -> String {
        let header = "req:";
        header.to_string() + &self.as_string()
    }

    pub fn parse(payload: &str) -> Result<Self, String> {
        println!("{}", payload);
        let header = "req:";
        if !payload.starts_with(header) {
            return Err("Header Missing".to_string());
        }
        Self::from_str(&payload[header.len()..])
    }
}