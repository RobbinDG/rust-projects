use crate::ServerRequest::ListQueues;
use std::str;
use std::str::Utf8Error;

pub struct Message {
    payload: String, // TODO byte string?
}

pub struct MessageQueue {
    pub messages: Vec<Message>,
}

#[derive(Debug)]
pub enum ServerRequest {
    ListQueues,
}

impl ServerRequest {
    pub fn as_str(&self) -> &str {
        match self {
            ListQueues => "list_queues",
        }
    }

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "list_queues" => Ok(ListQueues),
            _ => Err("Command not recognized".to_string()),
        }
    }

    pub fn as_payload(&self) -> String {
        let header = "req:";
        header.to_string() + self.as_str()
    }

    pub fn parse(payload: &str) -> Result<Self, String> {
        let header = "req:";
        if !payload.starts_with(header) {
            return Err("Header Missing".to_string());
        }
        Self::from_str(&payload[header.len()..])
    }
}

#[derive(Debug)]
pub struct ServerResponse {
    pub payload: String
}

impl ServerResponse {
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

impl MessageQueue {
    pub fn push(&mut self, message: Message) {
        self.messages.push(message)
    }

    pub fn pop(&mut self) -> Option<Message> {
        self.messages.pop()
    }
}


