#![allow(unused_imports)]
use std::{net::SocketAddr, time::Duration};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    select,
    sync::mpsc,
};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let listener = TcpListener::bind("0.0.0.0:8088").await;
    if let Err(_) = listener {
        eprintln!("fail to bind");
        return;
    }

    let listener = listener.unwrap();
    let (tx, mut rx) = mpsc::channel(32);

    for i in 0..10 {
        let tmp = i;
        let tx = tx.clone();
        tokio::task::spawn(async move {
            let connect = TcpStream::connect("127.0.0.1:8088").await.unwrap();
            let (mut rd, mut wr) = io::split(connect);
            let mut buf = vec![0; 32];

            if let Err(_) = wr.write_all(b"hello world\r\n").await {
                eprintln!("failed to write to socket")
            }

            match rd.read(&mut buf).await {
                Ok(n) if n == 0 => return,
                Ok(n) => {
                    println!("REP {}", String::from_utf8_lossy(&buf[..n]));
                }
                Err(_) => {
                    eprintln!("failed to read from socket");
                }
            }

            if tmp == 9 {
                if let Err(_) = tx.send(Command::Quit).await {
                    eprintln!("fail to send msg");
                    return;
                }
            } else {
                if let Err(_) = tx
                    .send(Command::Message {
                        key: String::from("ok\r\n"),
                    })
                    .await
                {
                    eprintln!("fail to send msg");
                    return;
                }
            }

            loop {
                tokio::time::sleep(Duration::from_millis(5000)).await;
                wr.write_all(b"hello world\r\n").await.unwrap();
            }
        });
    }

    loop {
        select! {
            Ok((socket, addr)) = listener.accept() => {
                tokio::task::spawn(async move {
                    process(socket, addr).await;
                });
            },
            Some(msg) = rx.recv() => {
                match msg {
                    Command::Quit => {
                        println!("bye bye\r\n");
                        //return;
                    },
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug)]
#[allow(unused)]
enum Command {
    Message { key: String },
    Quit,
}

async fn process(socket: TcpStream, addr: SocketAddr) {
    let (mut rd, mut wr) = io::split(socket);
    //let mut buf_reader = BufReader::new(rd);
    let mut buf = vec![0; 128];

    loop {
        match rd.read(&mut buf).await {
            Ok(n) if n > 0 => {
                wr.write_all(b"ok\r\n").await.unwrap();
                println!("{} GOT {:?}", addr, &buf[..n]);
            }
            Ok(_) => {
                return;
            }
            Err(_) => {
                eprintln!("failed to read from socket");
            }
        }
    }
}
