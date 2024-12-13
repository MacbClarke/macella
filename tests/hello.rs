use macella::{Request, Response, Server};

#[tokio::test]
async fn hello() {
    let _ = Server::new()
        .post("/hello", world)
        .bind("0.0.0.0:8080")
        .await;
}

async fn world(req: Request) -> Response {
    Response::ok(format!("hello {}", req.body().unwrap_or("")))
}
