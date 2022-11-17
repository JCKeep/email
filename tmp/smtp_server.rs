use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

#[tokio::main]
async fn main() {
    let smtp_listner = TcpListener::bind(("0.0.0.0", 25)).await.unwrap();

    loop {
        match smtp_listner.accept().await {
            Ok((stream, _)) => {
                tokio::task::spawn(async move {
                    handler(stream).await;
                });
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

async fn handler(mut stream: TcpStream) {
    stream
        .write(b"jckeep's smtp mail server\r\n")
        .await
        .unwrap();
    let mut buf = vec![0; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(n) if n == 0 => {
                println!("connection closed");
                break;
            }
            Ok(n) => {
                print!("{}", String::from_utf8_lossy(&buf[..n]));
                stream.write(b"250 ok\r\n").await.unwrap();
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}
