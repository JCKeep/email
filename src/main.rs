#![allow(unused_must_use)]
use std::{env, io::Write};

use email::{
    mime::{parse_content_type, Alternative, ContentTransferEncoding, ContentType},
    pop3::{pop3_handler_state, Pop3Builder, Pop3Command, Pop3UserState},
    smtp::SmtpBuilder,
};
use tokio::{
    io::{self, stdin, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

pub mod mime;
pub mod pop3;
pub mod smtp;

const CLEAR: &str = "\x1b[2J\x1b[H";

pub async fn pop3_handler(mut stream: TcpStream) {
    stream.write(b"+OK\r\n").await.unwrap();
    let (mut r, mut w) = io::split(stream);
    let mut state = Pop3UserState::new();
    let mut buf = vec![0; 1024];
    loop {
        match r.read(&mut buf).await {
            Ok(n) if n == 0 => {
                println!("connection closed");
                break;
            }
            Ok(n) => {
                if let Err(_) = pop3_handler_state(&mut w, &mut state, &buf, n).await {
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

fn parse_args(args: Vec<String>) -> Result<i32, ()> {
    if args.len() == 3 {
        if args[1].eq("-s") && args[2].eq("start") {
            return Ok(0);
        } else if args[1].eq("-t") {
            if args[2].eq("send") {
                return Ok(1);
            } else if args[2].eq("recv") {
                return Ok(2);
            }
        }
    }
    println!("usage: email -s start to run server or email -t send/recv");
    return Err(());
}

async fn send() -> Result<(), ()> {
    let mut rd = BufReader::new(stdin());
    let mut from = String::new();
    let mut to = String::new();
    let mut subject = String::new();
    let encoding = ContentTransferEncoding::Base64;
    let content_type = ContentType::MultipartMixed;
    let mut content = String::new();
    let mut attach = String::new();
    let mut attachment = Vec::new();
    let mut smtp = SmtpBuilder::new()
        .email("root@jckeep.top")
        .host("smtp.jckeep.top:25")
        .build()
        .await;
    print!("From: ");
    std::io::stdout().flush();
    rd.read_line(&mut from).await.unwrap();

    print!("To: ");
    std::io::stdout().flush();
    rd.read_line(&mut to).await.unwrap();

    print!("Subject: ");
    std::io::stdout().flush();
    rd.read_line(&mut subject).await.unwrap();

    println!("Content End with .<CR><CF>");
    std::io::stdout().flush();

    loop {
        rd.read_line(&mut content).await.unwrap();
        if content.ends_with(".\n") {
            break;
        }
    }

    attachment.push(Alternative {
        filename: None,
        content,
        content_type: ContentType::TextHtml,
        encoding: ContentTransferEncoding::Bit7,
    });

    print!("Attachment yes/no? ");
    std::io::stdout().flush();
    rd.read_line(&mut attach).await.unwrap();

    if attach.trim().to_lowercase().eq("yes") {
        println!("Enter filepath per line, end with .<CR><CF>");
        std::io::stdout().flush();
        loop {
            let mut file = String::new();
            rd.read_line(&mut file).await.unwrap();
            if file.ends_with(".\n") {
                break;
            }
            attachment.push(Alternative {
                filename: Some(String::from(file.trim())),
                content: "".to_string(),
                content_type: parse_content_type(file.trim()),
                encoding,
            })
        }
    } else {
        println!("You choose send a mail with no attachment");
    }

    println!("sending mail...");
    std::io::stdout().flush();
    match smtp.send(
        from.trim(),
        to.trim(),
        subject.trim(),
        ContentTransferEncoding::Bit7,
        content_type,
        "",
        Some(&attachment),
    )
    .await {
        Ok(_) => {
            println!("success");
        },
        Err(_) => {
            println!("file not exist");
        },
    }
    ;

    Ok(())
}

async fn recv() -> Result<(), ()> {
    let mut rd = BufReader::new(stdin());
    let mut username = String::new();

    print!("Enter your username: ");
    std::io::stdout().flush();
    rd.read_line(&mut username).await.unwrap();

    let mut pop = Pop3Builder::new()
        .email(&username.trim())
        .host("jckeep.top:110")
        .build()
        .await;

    let mails = pop.cmd(Pop3Command::INFO).await.unwrap();
    print!("{}", mails);
    std::io::stdout().flush();

    loop {
        print!("Enter mail ID to read detail# ");
        std::io::stdout().flush();
        let mut tmp_str = String::new();
        rd.read_line(&mut tmp_str).await.unwrap();
        let n: i32 = match tmp_str.trim().parse() {
            Ok(n) => n,
            Err(_) => {
                let ref_s = tmp_str.trim();
                if ref_s.to_lowercase().starts_with("quit") {
                    println!("\nBye");
                    return Ok(());
                } else if ref_s.starts_with("clear") {
                    print!("{}", CLEAR);
                    std::io::stdout().flush();
                    print!("{}", mails);
                    std::io::stdout().flush();
                    continue;
                }
                eprintln!("error input");
                continue;
            }
        };
        print!("{}", pop.cmd(Pop3Command::RETR(n)).await.unwrap());
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args: Vec<String> = env::args().collect();
    let rc = parse_args(args)?;

    if rc == 0 {
        let pop3_listner = TcpListener::bind(("0.0.0.0", 110)).await.unwrap();

        loop {
            match pop3_listner.accept().await {
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
    } else if rc == 1 {
        send().await?;
    } else if rc == 2 {
        recv().await?;
    } else {
        return Err(());
    }

    Ok(())
}
