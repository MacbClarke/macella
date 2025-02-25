use macella::{Request, Response, Server};

#[tokio::test]
async fn ping() {
    env_logger::init();
    let _ = Server::new().get("/ping", pong).bind("0.0.0.0:8080").await;
}

async fn pong(_: Request) -> Response {
    Response::ok("pong")
}
