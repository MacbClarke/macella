# 🚨🚧WIP🚧🚨

macella is an exercise for my personal learning of the http protocol and rust lang.

It is very sketchy and should not be used in a production environment.

I know the code sucks. 😔

If you have suggestions for improving it, they are most welcome!

# macella

A dead simple web server framework.

It currently provides:

- path routing
- basic get and post requests
- websocket

## Example

Ping:

```rust
use macella::{Request, Response, Server};

#[tokio::test]
async fn ping() {
    let _ = Server::new().get("/ping", pong).bind("0.0.0.0:8080").await;
}

async fn pong(_: Request) -> Response {
    Response::ok("pong")
}
```

Body data:

```rust
use macella::{Request, Response, Server};

#[tokio::test]
async fn hello() {
    let _ = Server::new()
        .post("/hello", world)
        .bind("0.0.0.0:8080")
        .await;
}

async fn world(req: Request) -> Response {
    Response::ok(format!(
        "hello {}",
        req.body_utf8().unwrap_or(Ok("")).unwrap()
    ))
}
```

Websocket:

```rust
use macella::Server;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::test]
async fn echo() {
    let _ = Server::new().ws("/ws", process).bind("0.0.0.0:8080").await;
}

async fn process(mut stream: TcpStream) -> () {
    let mut buf = [0; 1024];

    loop {
        let _ = stream.read(&mut buf).await.unwrap();

        let resp: [u8; 6] = [129, 4, 78, 77, 83, 76];
        stream.write_all(&resp).await.unwrap();
    }
}
```
