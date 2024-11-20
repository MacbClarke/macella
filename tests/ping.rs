use macella::Server;

#[tokio::test]
async fn ping() {
    let _ = Server::new().get("/ping", pong).bind("0.0.0.0:8888").await;
}

async fn pong(_: String) -> String {
    String::from("pong")
}
