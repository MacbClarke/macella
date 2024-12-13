use core::str;
use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use chrono::Local;
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    request::Request,
    responses::{Response, Status},
    server::{HttpHandler, WsHandler},
};

pub struct SocketBuffer<'a> {
    pub buffer: &'a [u8],
    pub length: usize,
}

impl<'a> SocketBuffer<'a> {
    pub fn new(buffer: &'a [u8], length: usize) -> SocketBuffer<'a> {
        SocketBuffer { buffer, length }
    }
}

pub struct Service {}

impl Service {
    pub fn new() -> Service {
        Service {}
    }

    fn ws_key_encode(&self, sec_key: &str) -> Result<String, ()> {
        let mut hasher = Sha1::new();

        hasher.update(format!("{sec_key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11").as_bytes());

        let key = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        Ok(key)
    }

    pub async fn process_socket(
        &mut self,
        mut socket: tokio::net::TcpStream,
        http_handlers: &Arc<HashMap<String, HttpHandler<Response>>>,
        ws_handlers: &Arc<HashMap<String, WsHandler<()>>>,
    ) -> Result<(), ()> {
        let mut buffer: [u8; 102400] = [0; 102400];

        loop {
            let n = socket.read(&mut buffer).await.unwrap();

            if n == 0 {
                return Ok(());
            }

            let req = Request::from(SocketBuffer::new(&buffer, n));

            let req_headers = req.header().unwrap_or(HashMap::new());

            #[cfg(debug_assertions)]
            println!("{} {}", req.method(), req.path());

            #[cfg(debug_assertions)]
            println!("{:?}", req.query());

            #[cfg(debug_assertions)]
            println!("{:?}", req.header());

            #[cfg(debug_assertions)]
            println!("{:?}", req.body());

            println!(
                "{} {} {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                req.method(),
                req.path()
            );

            if req_headers.get("Connection") == Some(&"Upgrade")
                && req_headers.get("Upgrade") == Some(&"websocket")
            {
                match ws_handlers.get(format!("WS{}", req.path()).as_str()) {
                    Some(handler) => {
                        let raw_key = req_headers.get("Sec-WebSocket-Key").unwrap();

                        let key = self.ws_key_encode(raw_key).unwrap();

                        let handshake = Response::new()
                            .status(Status::SWITCHING_PROTOCOLS)
                            .header("Connection", "Upgrade")
                            .header("Upgrade", "websocket")
                            .header("Sec-WebSocket-Accept", key);

                        if let Err(e) = socket.write_all(handshake.build().as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return Err(());
                        }

                        handler(socket).await;

                        return Ok(());
                    }
                    None => {
                        if let Err(e) = socket.write_all(Status::NOT_FOUND.as_bytes()).await {
                            eprintln!("failed to write to socket; err = {:?}", e);
                            return Err(());
                        }
                    }
                };

                return Ok(());
            }

            let connection_close_flag = req_headers.get("Connection") == Some(&"close");

            let resp = match http_handlers.get(format!("{}{}", req.method(), req.path()).as_str()) {
                Some(handler) => handler(req).await,
                None => Response::not_found(),
            };

            #[cfg(debug_assertions)]
            println!("{}", resp.build());

            if let Err(e) = socket.write_all(resp.build().as_bytes()).await {
                eprintln!("failed to write to socket; err = {:?}", e);
                return Err(());
            }

            if connection_close_flag {
                return Ok(());
            }
        }
    }
}
