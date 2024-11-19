use macella::Server;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};

#[tokio::test]
async fn echo() {
    let _ = Server::new().ws("/ws", &process).bind("0.0.0.0:8080").await;
}

async fn process(mut stream: TcpStream) -> () {
    let mut buf = [0; 1024];

    loop {
        let _n = stream.read(&mut buf).await.unwrap();

        let resp: [u8; 6] = [129, 4, 78, 77, 83, 76];
        stream.write_all(&resp).await.unwrap();
    }
}
