use macella::{Response, Server};

#[tokio::test]
async fn ping() {
    let _ = Server::new().get("/ping", pong).bind("0.0.0.0:8080").await;
}

async fn pong(_: String, _: String) -> Response {
    Response::ok("pong")
}
