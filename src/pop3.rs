#![allow(unused)]
use std::time::Duration;

use bytes::{Buf, BufMut};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader, WriteHalf},
    net::TcpStream,
    time::timeout,
};

#[derive(Debug, Clone)]
pub enum Pop3Command {
    LIST,
    INFO,
    USER(String),
    RETR(i32),
    TOP(i32, i32),
    DELE(i32),
    RSET,
    QUIT,
    NOOP,
}

#[derive(Debug)]
pub struct Pop3Client {
    email: Option<String>,
    password: Option<String>,
    host: Option<String>,
    upstream: Option<TcpStream>,
    buf: Vec<u8>,
    content_buffer: Vec<u8>,
}

impl Pop3Client {
    pub async fn cmd(&mut self, command: Pop3Command) -> Result<String, ()> {
        if self.upstream.is_some() {
            pop3_upstream_poll(self, command).await
        } else {
            match pop3_upstream_connect(self).await {
                Ok(c) => {
                    self.upstream = Some(c);
                    pop3_upstream_poll(self, command).await
                }
                Err(e) => {
                    eprintln!("{}", e);
                    Err(())
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Pop3UserState {
    pub user: Option<String>,
    pub file: Option<File>,
    pub buf: String,
    pub wbuf: Vec<u8>,
    pub mails: Vec<String>,
    pub froms: Vec<String>,
    pub times: Vec<String>,
    pub subjects: Vec<String>,
}

impl Pop3UserState {
    pub fn new() -> Self {
        Self {
            user: None,
            file: None,
            buf: String::new(),
            wbuf: vec![0; 4096],
            mails: Vec::new(),
            froms: Vec::new(),
            times: Vec::new(),
            subjects: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Pop3Builder {
    email: String,
    password: String,
    host: String,
}

impl Pop3Builder {
    pub fn new() -> Self {
        Self {
            email: String::new(),
            password: String::new(),
            host: String::new(),
        }
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = String::from(email);
        self
    }

    pub fn password(mut self, password: &str) -> Self {
        self.password = String::from(password);
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = String::from(host);
        self
    }

    pub async fn build(self) -> Pop3Client {
        let pw = if self.password.is_empty() {
            None
        } else {
            Some(self.password)
        };
        Pop3Client {
            email: Some(self.email),
            password: pw,
            host: Some(self.host),
            upstream: None,
            buf: vec![0; 4096],
            content_buffer: vec![0; 16 * 4096],
        }
    }
}

async fn pop3_upstream_connect(
    pop: &mut Pop3Client,
) -> Result<TcpStream, &'static str> {
    for _ in 0..5 {
        match timeout(Duration::from_millis(500), async {
            TcpStream::connect(pop.host.as_ref().unwrap())
                .await
                .unwrap()
        })
        .await
        {
            Ok(mut c) => {
                let buf = &mut pop.buf;
                c.read(buf).await.unwrap();
                if !buf.starts_with(b"+OK") {
                    return Err("connection refused");
                }
                c.write(
                    format!("USER {}\r\n", pop.email.as_ref().unwrap())
                        .as_bytes(),
                )
                .await
                .unwrap();
                c.read(buf).await.unwrap();
                if !buf.starts_with(b"+OK") {
                    return Err("email error");
                }
                if pop.password.is_some() {
                    c.write(
                        format!("PASS {}\r\n", pop.password.as_ref().unwrap())
                            .as_bytes(),
                    )
                    .await
                    .unwrap();
                    c.read(buf).await.unwrap();
                    if !buf.starts_with(b"+OK") {
                        return Err("password error");
                    }
                }
                return Ok(c);
            }
            Err(_) => {
                eprintln!("timeout");
            }
        }
    }
    eprintln!("timeout 5 times");
    Err("5 times timeout")
}

async fn pop3_upstream_poll(
    pop: &mut Pop3Client,
    cmd: Pop3Command,
) -> Result<String, ()> {
    let mut buf = vec![0; 1024];
    let mut c = pop.upstream.as_mut().unwrap();
    c.write(b"NOOP\r\n").await.unwrap();
    match c.read(&mut pop.buf).await {
        Ok(n) if n > 0 => {}
        _ => {
            pop3_upstream_connect(pop).await.unwrap();
            c = pop.upstream.as_mut().unwrap();
        }
    }

    match cmd {
        Pop3Command::LIST => {
            c.write(b"LIST\r\n").await.unwrap();
        }
        Pop3Command::INFO => {
            c.write(b"INFO\r\n").await.unwrap();
        }
        Pop3Command::TOP(msg, n) => {
            c.write(format!("TOP {} {}\r\n", msg, n).as_bytes())
                .await
                .unwrap();
        }
        Pop3Command::RETR(msg) => {
            c.write(format!("RETR {}\r\n", msg).as_bytes())
                .await
                .unwrap();
        }
        Pop3Command::DELE(msg) => {
            c.write(format!("DELE {}\r\n", msg).as_bytes())
                .await
                .unwrap();
            pop3_upstream_readline(pop).await?;
            if pop.buf.starts_with(b"+Ok") {
                return Ok(String::new());
            } else {
                return Err(());
            }
        }
        Pop3Command::QUIT => {
            c.write(b"QUIT\r\n").await.unwrap();
            pop.upstream = None;
            return Ok(String::new());
        }
        Pop3Command::RSET => {
            c.write(b"RSET\r\n").await.unwrap();
            pop3_upstream_readline(pop).await?;
            return Ok(String::new());
        }
        Pop3Command::NOOP => {
            c.write(b"NOOP\r\n").await.unwrap();
            pop3_upstream_readline(pop).await?;
            if pop.buf.starts_with(b"+Ok") {
                return Ok(String::new());
            } else {
                return Err(());
            }
        }
        Pop3Command::USER(u) => {
            c.write(format!("USER {}\r\n", u).as_bytes()).await.unwrap();
            pop3_upstream_readline(pop).await?;
            if pop.buf.starts_with(b"+Ok") {
                return Ok(String::new());
            } else {
                return Err(());
            }
        }
        _ => {
            unimplemented!()
        }
    }
    pop3_upstream_read_content(pop, cmd).await
}

async fn pop3_upstream_read_content(
    pop: &mut Pop3Client,
    cmd: Pop3Command,
) -> Result<String, ()> {
    let connect = pop.upstream.as_mut().unwrap();
    let buf = &mut pop.buf;
    let mut st = String::new();

    loop {
        match connect.read(buf).await {
            Ok(n) if n == 0 => {
                break Ok(st);
            }
            Ok(n) => {
                let s = String::from_utf8_lossy(&buf[..n]);
                st.push_str(&format!("{}", s));
                if s.ends_with(".\r\n") {
                    break Ok(st);
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                break Err(());
            }
        }
    }
}

async fn pop3_upstream_readline(pop: &mut Pop3Client) -> Result<(), ()> {
    pop.upstream
        .as_mut()
        .unwrap()
        .read(&mut pop.buf)
        .await
        .unwrap();
    Ok(())
}

pub async fn pop3_handler_state(
    w: &mut WriteHalf<TcpStream>,
    state: &mut Pop3UserState,
    buf: &[u8],
    n: usize,
) -> Result<(), ()> {
    match pop3_parse_command(buf, n).unwrap() {
        Pop3Command::INFO => {
            if state.user.is_none() {
                return Err(());
            }
            let mut info_buf = String::new();
            info_buf.push_str(&format!(
                "No   {:<20}  {:<30}  {:<15}\r\n",
                "From", "Time", "Subject"
            ));
            for i in 0..state.mails.len() {
                info_buf.push_str(&format!(
                    "{:<4} {:<20}  {:<30}  {:<15}\r\n",
                    i, state.froms[i], state.times[i], state.subjects[i]
                ));
            }
            info_buf.push_str(".\r\n");
            w.write_all(info_buf.as_bytes()).await.unwrap();
            Ok(())
        }
        Pop3Command::LIST => {
            if state.user.is_none() {
                return Err(());
            }
            let mut index = 0;
            let mut tmp_buf = String::new();
            for mail in &state.mails {
                tmp_buf
                    .push_str(format!("{} {}\r\n", index, mail.len()).as_str());
                index += 1;
            }
            tmp_buf.push_str(".\r\n");
            w.write_all(tmp_buf.as_bytes()).await.unwrap();
            Ok(())
        }
        Pop3Command::RETR(msg) => {
            if state.user.is_none() {
                return Err(());
            }
            let tmp_buf = format!("{}\r\n.\r\n", state.mails[msg as usize]);
            w.write_all(tmp_buf.as_bytes()).await.unwrap();
            Ok(())
        }
        Pop3Command::TOP(msg, _) => {
            if state.user.is_none() {
                return Err(());
            }
            let tmp_buf = format!("{}\r\n.\r\n", state.mails[msg as usize]);
            w.write_all(tmp_buf.as_bytes()).await.unwrap();
            Ok(())
        }
        Pop3Command::DELE(_) => {
            if state.user.is_none() {
                return Err(());
            }
            w.write(b"+OK\r\n").await.unwrap();
            Ok(())
        }
        Pop3Command::RSET => {
            if state.user.is_none() {
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
            state.user = Some(u);
            state.file = match File::open(format!(
                "/var/mail/{}",
                state.user.as_ref().unwrap()
            ))
            .await
            {
                Ok(f) => Some(f),
                Err(_) => return Err(()),
            };
            state
                .file
                .as_mut()
                .unwrap()
                .read_to_string(&mut state.buf)
                .await
                .unwrap();
            for s in state.buf.split("From ") {
                let mut start = 0;
                let end = s.len();
                for i in s.as_bytes() {
                    start += 1;
                    if *i == '\n' as u8 {
                        break;
                    }
                }
                if start == 0 {
                    continue;
                }
                let tmp_str = s[..start].to_string();
                let mut p = 0;
                for i in tmp_str.as_bytes() {
                    p += 1;
                    if *i == ' ' as u8 {
                        break;
                    }
                }
                let mail = s[start..end].to_string();
                if !mail.is_empty() {
                    if let Some(pt) = mail.as_str().find("Subject: ") {
                        let tmp_s = mail.as_bytes();
                        let mut ppt = pt + 9;
                        for i in pt + 9..tmp_s.len() {
                            ppt += 1;
                            if tmp_s[i] == '\n' as u8 {
                                break;
                            }
                        }
                        state.subjects.push(mail[pt + 9..ppt - 1].to_string());
                    }
                    state.mails.push(mail);
                    state.froms.push(tmp_str[..p - 1].to_string());
                    state.times.push(tmp_str[p + 1..start - 1].to_string());
                }
            }
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
    } else if buf.starts_with(b"INFO") {
        return Ok(Pop3Command::INFO);
    } else if buf.starts_with(b"USER") {
        let mut s = Vec::new();
        for i in 5..n {
            let tmp = buf[i];
            if (tmp > 'a' as u8 && tmp < 'z' as u8)
                || (tmp > 'A' as u8 && tmp < 'Z' as u8)
            {
                s.put_u8(tmp);
            }
        }
        return Ok(Pop3Command::USER(String::from_utf8(s).unwrap()));
    } else {
        return Ok(Pop3Command::QUIT);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn pop3_test() {
        let mut pop = Pop3Builder::new()
            .email("test")
            // .password("fhbwysogaphydiga")
            .host("localhost:110")
            .build()
            .await;

        println!("{}", pop.cmd(Pop3Command::LIST).await.unwrap());
        // pop.cmd(Pop3Command::USER("root".to_string())).await.unwrap();
        // pop.cmd(Pop3Command::RETR(115)).await.unwrap();
    }
}
