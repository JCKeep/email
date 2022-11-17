#![allow(unused)]
use std::time::Duration;

use bytes::{BufMut, BytesMut};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{tcp::ReadHalf, TcpStream},
    time::timeout,
};

#[tokio::main]
async fn main() {
    match timeout(Duration::from_secs(3), async {
        TcpStream::connect("smtp.qq.com:25").await.unwrap()
    })
    .await
    {
        Ok(c) => {
            timeout(Duration::from_secs(5), smtp_send(c)).await.unwrap();
        }
        Err(_) => {
            eprintln!("connection timeout");
        }
    }
}

async fn smtp_send(connection: TcpStream) {
    let (mut rd, mut wr) = io::split(connection);
    let mut buf = vec![0; 1024];
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));
    wr.write(b"HELO localhost\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"AUTH LOGIN\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"MjQwNzAxODM3MUBxcS5jb20=\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"Zmhid3lzb2dhcGh5ZGlnYQ==\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"MAIL FROM: <2407018371@qq.com>\r\n")
        .await
        .unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"RCPT TO: <monster_t@foxmail.com>\r\n")
        .await
        .unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"DATA\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"From: 2407018371@qq.com\r\nTo: monster_t@foxmail.com\r\nSubject: test\r\nMIME-Version: 1.0\r\nContent-Transfer-Encoding: base64\r\nContent-Type: text/html\r\n\r\n").await.unwrap();
    wr.write(HTML).await.unwrap();
    wr.write(b"\r\n.\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    wr.write(b"QUIT\r\n").await.unwrap();
    let n = rd.read(&mut buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));
}

const HTML: &[u8] = b"
PCFET0NUWVBFIGh0bWw+CjxodG1sPgo8aGVhZD4KPHRpdGxlPldlbGNvbWUgdG8gbmdpbnghPC90
aXRsZT4KPHN0eWxlPgogICAgYm9keSB7CiAgICAgICAgd2lkdGg6IDM1ZW07CiAgICAgICAgbWFy
Z2luOiAwIGF1dG87CiAgICAgICAgZm9udC1mYW1pbHk6IFRhaG9tYSwgVmVyZGFuYSwgQXJpYWws
IHNhbnMtc2VyaWY7CiAgICB9Cjwvc3R5bGU+CjwvaGVhZD4KPGJvZHk+CjxoMT5XZWxjb21lIHRv
IG5naW54ITwvaDE+CjxwPklmIHlvdSBzZWUgdGhpcyBwYWdlLCB0aGUgbmdpbnggd2ViIHNlcnZl
ciBpcyBzdWNjZXNzZnVsbHkgaW5zdGFsbGVkIGFuZAp3b3JraW5nLiBGdXJ0aGVyIGNvbmZpZ3Vy
YXRpb24gaXMgcmVxdWlyZWQuPC9wPgoKPHA+Rm9yIG9ubGluZSBkb2N1bWVudGF0aW9uIGFuZCBz
dXBwb3J0IHBsZWFzZSByZWZlciB0bwo8YSBocmVmPSJodHRwOi8vbmdpbngub3JnLyI+bmdpbngu
b3JnPC9hPi48YnIvPgpDb21tZXJjaWFsIHN1cHBvcnQgaXMgYXZhaWxhYmxlIGF0CjxhIGhyZWY9
Imh0dHA6Ly9uZ2lueC5jb20vIj5uZ2lueC5jb208L2E+LjwvcD4KCjxwPjxlbT5UaGFuayB5b3Ug
Zm9yIHVzaW5nIG5naW54LjwvZW0+PC9wPgo8L2JvZHk+CjwvaHRtbD4K
";
