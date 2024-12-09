use core::str;
use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use chrono::Local;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::server::{HttpHandler, WsHandler};

struct SocketBuffer {
    buffer: [u8; 102400],
    length: usize,
}

impl SocketBuffer {
    pub fn new() -> SocketBuffer {
        SocketBuffer {
            buffer: [0; 102400],
            length: 0,
        }
    }
}

pub struct Service {
    socket_buffer: SocketBuffer,
}

impl Service {
    pub fn new() -> Service {
        Service {
            socket_buffer: SocketBuffer::new(),
        }
    }
    fn decode_req<'a>(&'a self) -> Result<(&'a str, HashMap<&'a str, &'a str>, String), ()> {
        let req = str::from_utf8(&self.socket_buffer.buffer[..self.socket_buffer.length]).unwrap();

        let mut headers: HashMap<&str, &str> = HashMap::new();

        let mut body_raw: String = String::new();

        let mut lines = req.lines();

        let lead = lines.next();

        let mut body_flag = false;

        for line in lines {
            if line.is_empty() {
                body_flag = true;
                continue;
            }

            if !body_flag {
                let split = line.split_once(": ");
                headers.insert(split.unwrap().0, split.unwrap().1);
            } else {
                body_raw.push_str(line);
            }
        }

        let lead_str = lead.unwrap();

        Ok((lead_str, headers, body_raw))
    }

    fn ws_key_encode<'a>(&self, sec_key: &'a str) -> Result<String, ()> {
        let mut hasher = Sha1::new();

        hasher.update(format!("{sec_key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11").as_bytes());

        let key = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        Ok(key)
    }

    pub async fn process_socket(
        &mut self,
        mut socket: tokio::net::TcpStream,
        http_handlers: &Arc<HashMap<String, HttpHandler<String>>>,
        ws_handlers: &Arc<HashMap<String, WsHandler<()>>>,
    ) -> Result<(), ()> {
        loop {
            let n = socket.read(&mut self.socket_buffer.buffer).await.unwrap();

            self.socket_buffer.length = n;

            if n == 0 {
                return Ok(());
            }

            let (leads, headers, body_raw) = self.decode_req()?;

            #[cfg(debug_assertions)]
            println!("{}", leads);

            #[cfg(debug_assertions)]
            println!("{:?}", headers);

            #[cfg(debug_assertions)]
            println!("{}", body_raw);

            let leads_split: Vec<&str> = leads.splitn(3, ' ').collect();

            let method = leads_split[0];

            let uri = leads_split[1];

            let uri_split: Vec<&str> = uri.splitn(2, '?').collect();

            let path = uri_split[0];

            let query = uri_split.get(1).cloned().unwrap_or("");

            println!(
                "{} {} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                method,
                uri
            );

            #[cfg(debug_assertions)]
            println!("{:?}", query);

            #[cfg(debug_assertions)]
            println!("{}", method);

            #[cfg(debug_assertions)]
            println!("{}", path);

            if headers.get("Connection") == Some(&"Upgrade")
                && headers.get("Upgrade") == Some(&"websocket")
            {
                match ws_handlers.get(format!("WS{}", path).as_str()) {
                    Some(handler) => {
                        let raw_key = headers.get("Sec-WebSocket-Key").unwrap();

                        let key = self.ws_key_encode(raw_key).unwrap();

                        let handshake = format!("HTTP/1.1 101 Switching Protocols\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Accept: {key}\r\n\r\n");

                        if let Err(e) = socket.write_all(handshake.as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return Err(());
                        }

                        handler(socket).await;

                        return Ok(());
                    }
                    None => {
                        if let Err(e) = socket.write_all(b"HTTP/1.1 404 Not Found").await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return Err(());
                        }
                    }
                };

                return Ok(());
            }

            let (status, content) = match http_handlers.get(format!("{}{}", method, path).as_str())
            {
                Some(handler) => ("200 OK", handler(query.to_string(), body_raw).await),
                None => ("404 Not Found", String::new()),
            };

            let content_length = content.len();

            let response = format!("HTTP/1.1 {status}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Credentials: true\r\nContent-Length: {content_length}\r\nContent-Type: application/json\r\n\r\n{content}");

            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("failed to write to socket; err = {:?}", e);
                return Err(());
            }

            if headers.get("Connection") == Some(&"close") {
                return Ok(());
            }
        }
    }
}
