use std::collections::HashMap;

use super::Status;

pub struct Response {
    status: String,
    headers: HashMap<String, String>,
    body: String,
}

impl Default for Response {
    fn default() -> Self {
        Self::new()
    }
}

impl Response {
    pub fn build(&self) -> String {
        let lead = format!("HTTP/1.1 {}", self.status);

        let mut headers = String::new();

        for (k, v) in &self.headers {
            headers += &format!("{}: {}\r\n", k, v);
        }

        headers += &format!("Content-Length: {}\r\n", self.body.len());

        format!("{lead}\r\n{headers}\r\n{}", self.body)
    }

    pub fn status<T: AsRef<str>>(mut self, status: T) -> Self {
        self.status = status.as_ref().to_string();
        self
    }

    pub fn header<T: AsRef<str>, Y: AsRef<str>>(mut self, key: T, value: Y) -> Self {
        self.headers
            .insert(key.as_ref().to_string(), value.as_ref().to_string());
        self
    }

    pub fn body<T: AsRef<str>>(mut self, body: T) -> Self {
        self.body = body.as_ref().to_string();
        self
    }

    pub fn new() -> Self {
        Self {
            status: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn ok<T: AsRef<str>>(data: T) -> Self {
        Self {
            status: String::from(Status::OK),
            headers: HashMap::new(),
            body: data.as_ref().to_string(),
        }
    }

    pub fn not_found() -> Self {
        Self {
            status: String::from(Status::NOT_FOUND),
            headers: HashMap::new(),
            body: String::new(),
        }
    }

    pub fn err<T: AsRef<str>>(data: T) -> Self {
        Self {
            status: String::from(Status::INTERNAL_SERVER_ERROR),
            headers: HashMap::new(),
            body: data.as_ref().to_string(),
        }
    }
}
