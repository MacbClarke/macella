use macella::Server;

#[tokio::test]
async fn ping() {
    let _ = Server::new()
        .get("/ping", |_| String::from("pong"))
        .bind("0.0.0.0:8888")
        .await;
}

