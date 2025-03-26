use core::str;
use std::{collections::HashMap, sync::Arc};

use base64::Engine;
use log::{debug, error, info, warn};
use sha1::{Digest, Sha1};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{
    request::{ParseError, Request}, // Import ParseError
    responses::{Response, Status},
    server::{HttpHandler, WsHandler},
};

// Remove SocketBuffer as it's no longer used directly for request parsing
// pub struct SocketBuffer<'a> { ... }
// impl<'a> SocketBuffer<'a> { ... }

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
        // Use a Vec<u8> as a persistent read buffer for the connection
        let mut read_buf = Vec::with_capacity(8 * 1024); // Start with 8KB capacity
        let mut socket_closed = false;

        loop {
            // Only read more data if the socket is not closed
            if !socket_closed {
                // Create a temporary buffer for reading new data
                let mut temp_buf = [0; 4096]; // Read in chunks
                let n = match socket.read(&mut temp_buf).await {
                Ok(n) => n,
                Err(e) => {
                        warn!("Error reading from socket: {:?}", e);
                        0 // Treat error as connection closed for simplicity here
                    }
                };

                if n == 0 {
                    // Client closed the connection
                    info!("Client disconnected.");
                    socket_closed = true;
                } else {
                    // Append new data to the persistent buffer
                    read_buf.extend_from_slice(&temp_buf[..n]);
                }
            }

            // --- Request Parsing and Processing Loop ---
            // Try to parse requests from the buffer as long as possible
            loop {
                match Request::parse(&read_buf) {
                    Ok((req, consumed_bytes)) => {
                        // Successfully parsed a request
                        let req_headers = req.header().unwrap_or_default();
                        debug!("Parsed request: {} {}", req.method(), req.path());

                        info!("Processing request: {} {}", req.method(), req.path());

                        // --- WebSocket Upgrade Handling ---
                        if req_headers.get("Connection").map(|v| v.to_lowercase()) == Some("upgrade".to_string())
                            && req_headers.get("Upgrade").map(|v| v.to_lowercase()) == Some("websocket".to_string())
                        {
                            match ws_handlers.get(format!("WS{}", req.path()).as_str()) {
                                Some(handler) => {
                                    let raw_key = match req_headers.get("Sec-WebSocket-Key") {
                                        Some(k) => k,
                                        None => {
                                            error!("Missing Sec-WebSocket-Key for WebSocket upgrade");
                                            // Send Bad Request? Or just close? Let's close for now.
                                            // Remove consumed bytes before returning
                                            read_buf.drain(..consumed_bytes);
                                            return Err(());
                                        }
                                    };

                                    let key = match self.ws_key_encode(raw_key) {
                                        Ok(k) => k,
                                        Err(_) => {
                                            error!("Failed to encode WebSocket key");
                                            read_buf.drain(..consumed_bytes);
                                            return Err(()); // Internal server error
                                        }
                                    };

                                    let handshake = Response::new()
                                        .status(Status::SWITCHING_PROTOCOLS)
                                        .header("Connection", "Upgrade")
                                        .header("Upgrade", "websocket")
                                        .header("Sec-WebSocket-Accept", key);

                                    if let Err(e) = socket.write_all(handshake.build().as_bytes()).await {
                                        error!("Failed to write WebSocket handshake; err = {:?}", e);
                                        // Don't remove consumed bytes here, as the connection is broken
                                        return Err(());
                                    }

                                    info!("WebSocket connection established for {}", req.path());
                                    // WebSocket handler takes over the socket. We are done with this connection.
                                    // Before handing over, remove the consumed request bytes from the buffer
                                    // (though it shouldn't matter much as the connection is changing protocol).
                                    read_buf.drain(..consumed_bytes);
                                    handler(socket).await;
                                    return Ok(()); // Successfully handled WebSocket upgrade
                                }
                                None => {
                                    // WebSocket requested for a path with no handler
                                    warn!("WebSocket upgrade requested for unknown path: {}", req.path());
                                    let resp = Response::not_found(); // Or maybe 404 specifically for WS?
                                    if let Err(e) = socket.write_all(resp.build().as_bytes()).await {
                                        error!("Failed to write 404 response for WS; err = {:?}", e);
                                        return Err(());
                                    }
                                    // Remove consumed bytes and continue processing (or close if Connection: close)
                                    read_buf.drain(..consumed_bytes);
                                    // Check if this failed WS request also asked to close
                                    if req_headers.get("Connection").map(|v| v.to_lowercase()) == Some("close".to_string()) {
                                        info!("Closing connection after failed WS upgrade request.");
                                        return Ok(());
                                    }
                                    continue; // Continue the inner loop for potential pipelined requests
                                }
                            }
                        }

                        // --- Regular HTTP Request Handling ---
                        let connection_close_flag = req_headers.get("Connection").map(|v| v.to_lowercase()) == Some("close".to_string());

                        let resp = match http_handlers.get(format!("{}{}", req.method(), req.path()).as_str()) {
                            Some(handler) => handler(req).await,
                            None => Response::not_found(),
                        };

                        debug!("Sending response:\n{}", resp.build());

                        if let Err(e) = socket.write_all(resp.build().as_bytes()).await {
                            error!("Failed to write response; err = {:?}", e);
                            // Don't remove consumed bytes, connection is broken
                            return Err(());
                        }

                        // Remove the processed request bytes from the buffer
                        read_buf.drain(..consumed_bytes);

                        // If Connection: close was requested, stop processing more requests and close
                        if connection_close_flag {
                            info!("Closing connection due to 'Connection: close' header.");
                            return Ok(());
                        }

                        // Continue inner loop to check for more requests in the buffer
                    }
                    Err(ParseError::Incomplete) => {
                        // Not enough data in the buffer to parse a full request
                        // Break the inner loop and wait for more data from the socket
                        break;
                    }
                    Err(ParseError::InvalidFormat(reason)) => {
                        // Invalid request format
                        error!("Invalid request format: {}", reason);
                        // Send a 400 Bad Request response
                        let resp = Response::err("Bad Request").status(Status::BAD_REQUEST); // Assuming Response::err sets 500, override status
                        if let Err(e) = socket.write_all(resp.build().as_bytes()).await {
                            error!("Failed to write 400 Bad Request response; err = {:?}", e);
                        }
                        // Close the connection immediately after sending 400
                        return Err(());
                    }
                }
            } // End of inner request processing loop

            // If the socket is closed and the buffer is empty, we are done
            if socket_closed && read_buf.is_empty() {
                info!("Connection closed and buffer empty. Shutting down service loop.");
                return Ok(());
            }

            // If the socket is closed but there's still data, it means we have an incomplete request left.
            if socket_closed && !read_buf.is_empty() {
                warn!("Connection closed with incomplete request data in buffer ({} bytes).", read_buf.len());
                return Err(()); // Or Ok(()) depending on desired behavior for partial requests on close
            }

            // If we reached here, it means ParseError::Incomplete occurred,
            // and the socket is not closed yet. Continue the outer loop to read more data.

        } // End of outer socket read loop
    }
}
