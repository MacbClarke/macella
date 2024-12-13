use core::{panic, str};
use std::collections::HashMap;

use crate::service::SocketBuffer;

pub struct Request {
    header: Option<String>,
    method: String,
    path: String,
    query: Option<String>,
    body: Option<String>,
}

impl Request {
    pub fn header_raw(&self) -> Option<&str> {
        self.header.as_deref()
    }

    pub fn header(&self) -> Option<HashMap<&str, &str>> {
        self.header.as_ref()?;

        let mut map: HashMap<&str, &str> = HashMap::new();

        let h = self.header.as_deref();

        for qry in h.unwrap().split("\r\n") {
            if let Some((k, v)) = qry.split_once(": ") {
                map.insert(k, v);
            }
        }

        Some(map)
    }

    pub fn method(&self) -> &str {
        self.method.as_ref()
    }

    pub fn path(&self) -> &str {
        self.path.as_ref()
    }

    pub fn query_raw(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn query(&self) -> Option<HashMap<&str, &str>> {
        self.query.as_ref()?;

        let mut map: HashMap<&str, &str> = HashMap::new();

        let q = self.query.as_deref();

        for qry in q.unwrap().split("&") {
            if let Some((k, v)) = qry.split_once("=") {
                map.insert(k, v);
            }
        }

        Some(map)
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }
}

impl<'a> From<SocketBuffer<'a>> for Request {
    fn from(value: SocketBuffer<'a>) -> Self {
        let str = match str::from_utf8(&value.buffer[..value.length]) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };

        let (request_line, headers_and_body) = str.split_once("\r\n").unwrap();
        let (headers, body) = headers_and_body.split_once("\r\n\r\n").unwrap();

        let request_parts: Vec<&str> = request_line.split_whitespace().collect();
        if request_parts.len() != 3 {
            panic!("Invalid request line format");
        }

        let (path, query) = match request_parts[1].split_once("?") {
            Some((p, q)) => (p, Some(q.to_string())),
            None => (request_parts[1], None),
        };

        let mut option_headers: Option<String> = None;
        if !headers.is_empty() {
            option_headers = Some(headers.to_string());
        }

        let mut option_body: Option<String> = None;
        if !body.is_empty() {
            option_body = Some(body.to_string());
        }

        Self {
            header: option_headers,
            method: request_parts[0].to_string(),
            path: path.to_string(),
            query,
            body: option_body,
        }
    }
}
