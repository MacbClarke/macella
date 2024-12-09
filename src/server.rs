use core::str;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use tokio::net::TcpListener;

use crate::service::Service;

pub type HttpHandler<O> =
    Arc<dyn Fn(String, String) -> Pin<Box<dyn Future<Output = O> + Send>> + Send + Sync>;
pub type WsHandler<O> =
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
                if let Err(e) = Service::new()
                    .process_socket(socket, &http_handlers, &ws_handlers)
                    .await
                {
                    eprintln!("Error processing socket: {:?}", e)
                }
            });
        }
    }
}
