use core::str;
use std::{collections::HashMap, str::Utf8Error};

// Define potential errors during request parsing
#[derive(Debug, PartialEq)]
pub enum ParseError {
    Incomplete,
    InvalidFormat(String), // Include a reason for the invalid format
}

pub struct Request {
    header: Option<String>,
    method: String,
    path: String,
    query: Option<String>,
    body: Option<Vec<u8>>,
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

    pub fn body(&self) -> Option<&[u8]> {
        self.body.as_deref()
    }

    pub fn body_utf8(&self) -> Option<Result<&str, Utf8Error>> {
        self.body.as_ref()?;

        let b = self.body.as_ref().unwrap();

        Some(str::from_utf8(b))
    }

    // New function to parse a request from a buffer slice
    // Returns the parsed request and the number of bytes consumed from the buffer
    pub fn parse(buffer: &[u8]) -> Result<(Self, usize), ParseError> {
        // Find the end of the headers (\r\n\r\n)
        let header_end_pos = buffer
            .windows(4)
            .position(|w| w == b"\r\n\r\n")
            .ok_or(ParseError::Incomplete)?;

        let header_data = &buffer[..header_end_pos];
        let header_str = str::from_utf8(header_data)
            .map_err(|e| ParseError::InvalidFormat(format!("Invalid UTF-8 in headers: {}", e)))?;

        let (request_line, headers_part) = header_str
            .split_once("\r\n")
            .ok_or_else(|| ParseError::InvalidFormat("Missing request line".to_string()))?;

        // Parse Request Line
        let request_parts: Vec<&str> = request_line.split_whitespace().collect();
        if request_parts.len() != 3 {
            return Err(ParseError::InvalidFormat(format!(
                "Invalid request line format: {}",
                request_line
            )));
        }
        let method = request_parts[0].to_string();
        let full_path = request_parts[1];
        // TODO: Validate HTTP version request_parts[2] if needed

        // Parse Path and Query
        let (path, query) = match full_path.split_once("?") {
            Some((p, q)) => (p.to_string(), Some(q.to_string())),
            None => (full_path.to_string(), None),
        };

        // Store raw headers string
        let option_headers = if headers_part.is_empty() {
            None
        } else {
            Some(headers_part.to_string())
        };

        // --- Body Parsing ---
        let headers_map = Self::parse_headers_str(headers_part);
        let content_length = headers_map
            .get("Content-Length")
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let header_size = header_end_pos + 4; // Include the \r\n\r\n
        let total_request_size = header_size + content_length;

        // Check if the buffer contains the complete request (headers + body)
        if buffer.len() < total_request_size {
            return Err(ParseError::Incomplete);
        }

        let body_data = &buffer[header_size..total_request_size];

        let option_body = if body_data.is_empty() {
            None
        } else {
            Some(body_data.to_vec())
        };

        Ok((
            Self {
                header: option_headers,
                method,
                path,
                query,
                body: option_body,
            },
            total_request_size, // Return the total bytes consumed by this request
        ))
    }

    // Helper function to parse header string into a HashMap (used internally by parse)
    // Note: This is similar to the existing `header` method but operates on a string slice.
    fn parse_headers_str(headers_part: &str) -> HashMap<&str, &str> {
        let mut map: HashMap<&str, &str> = HashMap::new();
        for line in headers_part.split("\r\n") {
            if let Some((k, v)) = line.split_once(": ") {
                map.insert(k.trim(), v.trim());
            }
        }
        map
    }
}

// Remove the old From<SocketBuffer> implementation
// impl<'a> From<SocketBuffer<'a>> for Request { ... }
