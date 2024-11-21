use core::str;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use base64::Engine;
use chrono::Local;
use sha1::{Digest, Sha1};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

type HttpHandler<O> =
    Arc<dyn Fn(String, String) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>;
type WsHandler<O> =
    Arc<dyn Fn(tokio::net::TcpStream) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>;

pub struct Server {
    http_handlers: HashMap<String, HttpHandler<String>>,
    ws_handlers: HashMap<String, WsHandler<()>>,
}

impl Server {
    pub fn new() -> Server {
        Server {
            http_handlers: HashMap::new(),
            ws_handlers: HashMap::new(),
        }
    }
    pub fn get<F, U>(&mut self, route: &'static str, handler: F) -> &mut Server
    where
        F: Fn(String, String) -> U + Send + Sync + 'static,
        U: Future<Output = String> + Send + 'static,
    {
        self.http_handlers.insert(
            format!("GET{}", route),
            Arc::new(move |a, b| Box::pin(handler(a, b))),
        );
        self
    }
    pub fn post<F, U>(&mut self, route: &'static str, handler: F) -> &mut Server
    where
        F: Fn(String, String) -> U + Send + Sync + 'static,
        U: Future<Output = String> + Send + 'static,
    {
        self.http_handlers.insert(
            format!("POST{}", route),
            Arc::new(move |a, b| Box::pin(handler(a, b))),
        );
        self
    }
    pub fn ws<F, U>(&mut self, route: &'static str, handler: F) -> &mut Server
    where
        F: Fn(tokio::net::TcpStream) -> U + Send + Sync + 'static,
        U: Future<Output = ()> + Send + 'static,
    {
        self.ws_handlers.insert(
            format!("WS{}", route),
            Arc::new(move |a| Box::pin(handler(a))),
        );
        self
    }
    pub async fn bind(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        let http_handlers = Arc::new(self.http_handlers.clone());
        let ws_handlers = Arc::new(self.ws_handlers.clone());

        loop {
            let (socket, _) = listener.accept().await?;
            let http_handlers = Arc::clone(&http_handlers);
            let ws_handlers = Arc::clone(&ws_handlers);
            tokio::spawn(async move {
                if let Err(e) = process_socket(socket, &http_handlers, &ws_handlers).await {
                    eprintln!("Error processing socket: {:?}", e)
                }
            });
        }
    }
}

fn decode_req<'a>(
    buf: &'a [u8; 102400],
    length: usize,
) -> Result<(&'a str, HashMap<&'a str, &'a str>, String), ()> {
    let req = str::from_utf8(&buf[..length]).unwrap();

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

fn ws_key_encode<'a>(sec_key: &'a str) -> Result<String, ()> {
    let mut hasher = Sha1::new();

    hasher.update(format!("{sec_key}258EAFA5-E914-47DA-95CA-C5AB0DC85B11").as_bytes());

    let key = base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

    Ok(key)
}

async fn process_socket(
    mut socket: tokio::net::TcpStream,
    http_handlers: &Arc<HashMap<String, HttpHandler<String>>>,
    ws_handlers: &Arc<HashMap<String, WsHandler<()>>>,
) -> Result<(), ()> {
    let mut buf = [0; 102400];

    loop {
        let n = socket.read(&mut buf).await.unwrap();

        if n == 0 {
            return Ok(());
        }

        let (leads, headers, body_raw) = decode_req(&buf, n)?;

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

                    let key = ws_key_encode(raw_key).unwrap();

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

        let (status, content) = match http_handlers.get(format!("{}{}", method, path).as_str()) {
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
