use bytes::BufMut;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, WriteHalf},
    net::{TcpListener, TcpStream},
};

use email::pop3::Pop3Command;

#[tokio::main]
async fn main() {
    let smtp_listner = TcpListener::bind(("0.0.0.0", 110)).await.unwrap();

    loop {
        match smtp_listner.accept().await {
            Ok((stream, _)) => {
                tokio::task::spawn(async move {
                    pop3_handler(stream).await;
                });
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
}

async fn pop3_handler(mut stream: TcpStream) {
    stream
        .write(b"jckeep's pop3 mail server\r\n")
        .await
        .unwrap();
    let (mut r, mut w) = io::split(stream);
    let mut user: Option<String> = None;
    let mut buf = vec![0; 1024];
    loop {
        match r.read(&mut buf).await {
            Ok(n) if n == 0 => {
                println!("connection closed");
                break;
            }
            Ok(n) => {
                if let Err(_) = pop3_handler_state(&mut w, &mut user, &buf, n).await {
                    return;
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break;
            }
        }
    }
}

async fn pop3_handler_state(
    w: &mut WriteHalf<TcpStream>,
    user: &mut Option<String>,
    buf: &[u8],
    n: usize,
) -> Result<(), ()> {
    match pop3_parse_command(buf, n).unwrap() {
        Pop3Command::LIST => {
            if user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::RETR(_) => {
            if user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::TOP(_, _) => {
            if user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::DELE(_) => {
            if user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::RSET => {
            if user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::QUIT => {
            w.write(b"+OK bye\r\n").await.unwrap();
            Err(())
        }
        Pop3Command::NOOP => {
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::USER(u) => {
            *user = Some(u);
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
    }
}

fn pop3_parse_command(buf: &[u8], n: usize) -> Result<Pop3Command, ()> {
    if buf.starts_with(b"LIST") {
        return Ok(Pop3Command::LIST);
    } else if buf.starts_with(b"RETR") {
        let mut num = 0;
        for i in 4..n {
            let tmp = buf[i] as i32;
            if tmp >= '0' as i32 && tmp <= '9' as i32 {
                num = num * 10 + tmp - '0' as i32;
            }
        }
        println!("{}", num);
        return Ok(Pop3Command::RETR(num));
    } else if buf.starts_with(b"DELE") {
        let mut num = 0;
        for i in 4..n {
            let tmp = buf[i] as i32;
            if tmp >= '0' as i32 && tmp <= '9' as i32 {
                num = num * 10 + tmp - '0' as i32;
            }
        }
        println!("{}", num);
        return Ok(Pop3Command::DELE(num));
    } else if buf.starts_with(b"TOP") {
        let mut num = 0;
        let mut num1 = 0;
        let mut index = 0;
        for i in 4..n {
            let tmp = buf[i] as i32;
            if tmp >= '0' as i32 && tmp <= '9' as i32 {
                num = num * 10 + tmp - '0' as i32;
            } else {
                index = i + 1;
                break;
            }
        }
        for i in index..n {
            let tmp = buf[i] as i32;
            if tmp >= '0' as i32 && tmp <= '9' as i32 {
                num1 = num1 * 10 + tmp - '0' as i32;
            } else {
                break;
            }
        }
        println!("{} {}", num, num1);
        return Ok(Pop3Command::TOP(num, num1));
    } else if buf.starts_with(b"RSET") {
        return Ok(Pop3Command::RSET);
    } else if buf.starts_with(b"NOOP") {
        return Ok(Pop3Command::NOOP);
    } else if buf.starts_with(b"USER") {
        let mut s = Vec::new();
        for i in 5..n {
            let tmp = buf[i];
            if (tmp > 'a' as u8 && tmp < 'z' as u8) || (tmp > 'A' as u8 && tmp < 'Z' as u8) {
                s.put_u8(tmp);
            }
        }
        return Ok(Pop3Command::USER(String::from_utf8(s).unwrap()));
    } else {
        return Ok(Pop3Command::QUIT);
    }
}
