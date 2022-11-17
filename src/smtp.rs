#![allow(unused)]
use std::time::Duration;

use base64::encode;
use bytes::BufMut;
use encoding::{EncoderTrap, Encoding};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

use crate::mime::{mime_encode, Alternative, ContentTransferEncoding, ContentType};

#[derive(Debug)]
pub struct SmtpClient {
    address: Option<String>,
    email: Option<String>,
    host: Option<String>,
    token: Option<String>,
    upstream: Option<TcpStream>,
    buf: Vec<u8>,
    sdata_buf: Vec<u8>,
}

impl SmtpClient {
    pub async fn send(
        &mut self,
        from: &str,
        to: &str,
        subject: &str,
        cte: ContentTransferEncoding,
        content_type: ContentType,
        content: &str,
        attach: Option<&[Alternative]>,
    ) -> Result<(), ()> {
        if self.upstream.is_none() {
            match smtp_upstream_connect(self).await {
                Ok(c) => {
                    self.upstream = Some(c);
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return Err(());
                }
            }
        }
        let mut t = vec![0; 16];
        let c = self.upstream.as_mut().unwrap();
        c.write(b"NOOP\r\n").await.unwrap();
        match c.read(&mut t).await {
            Ok(n) if (n > 0) && (t.starts_with(b"250")) => {}
            _ => {
                smtp_upstream_connect(self).await.unwrap();
            }
        }
        smtp_upstream_send(self, from, to, subject, cte, content_type, content, attach).await
    }

    pub async fn quit(mut self) {
        if let Some(mut c) = self.upstream {
            c.write(b"QUIT\r\n").await.unwrap();
        }
    }
}

#[derive(Debug)]
pub struct SmtpBuilder {
    address: String,
    email: String,
    token: String,
    host: String,
}

impl SmtpBuilder {
    pub fn new() -> Self {
        Self {
            address: String::new(),
            email: String::new(),
            token: String::new(),
            host: String::new(),
        }
    }

    pub fn email(mut self, email: &str) -> Self {
        self.email = encode(email);
        self
    }

    pub fn token(mut self, token: &str) -> Self {
        self.token = encode(token);
        self
    }

    pub fn address(mut self, addr: &str) -> Self {
        self.address = addr.to_string();
        self
    }

    pub fn host(mut self, host: &str) -> Self {
        self.host = host.to_string();
        self
    }

    pub async fn build(mut self) -> SmtpClient {
        if self.address.is_empty() {
            self.address = "localhost".to_string();
        }
        let t = if self.token.is_empty() {
            None
        } else {
            Some(self.token)
        };
        SmtpClient {
            address: Some(self.address),
            email: Some(self.email),
            token: t,
            host: Some(self.host),
            upstream: None,
            buf: vec![0; 4096],
            sdata_buf: vec![0; 4096],
        }
    }
}

async fn smtp_upstream_connect(smtp: &mut SmtpClient) -> Result<TcpStream, &'static str> {
    for _ in 0..5 {
        match timeout(Duration::from_millis(500), async {
            TcpStream::connect(smtp.host.as_ref().unwrap())
                .await
                .unwrap()
        })
        .await
        {
            Ok(mut c) => {
                let buf = &mut smtp.buf;
                c.read(buf).await.unwrap();
                if !buf.starts_with(b"220") {
                    return Err("connection refused");
                }
                c.write(format!("HELO {}\r\n", smtp.address.as_ref().unwrap()).as_bytes())
                    .await
                    .unwrap();
                loop {
                    match c.read(buf).await {
                        Ok(n) if n == 0 => {
                            break;
                        }
                        Ok(n) => {
                            if !buf.starts_with(b"250") {
                                return Err("hello error");
                            }
                            // if buf[..n].ends_with(b"250 OK\r\n") {
                            //     break;
                            // }
                            break;
                        }
                        Err(e) => {
                            eprintln!("{}", e);
                            return Err("error");
                        }
                    }
                }
                if smtp.token.is_some() {
                    c.write(b"AUTH LOGIN\r\n").await.unwrap();
                    c.read(buf).await.unwrap();
                    if !buf.starts_with(b"334") {
                        return Err("auth login error");
                    }
                    c.write(format!("{}\r\n", smtp.email.as_ref().unwrap()).as_bytes())
                        .await
                        .unwrap();
                    c.read(buf).await.unwrap();
                    if !buf.starts_with(b"334") {
                        return Err("auth login error2");
                    }
                    c.write(format!("{}\r\n", smtp.token.as_ref().unwrap()).as_bytes())
                        .await
                        .unwrap();
                    c.read(buf).await.unwrap();
                    if !buf.starts_with(b"235") {
                        return Err("authentication failure");
                    }
                }
                return Ok(c);
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    Err("timeout 5 times")
}

async fn smtp_upstream_send(
    smtp: &mut SmtpClient,
    from: &str,
    to: &str,
    subject: &str,
    content_transfer_encoding: ContentTransferEncoding,
    content_type: ContentType,
    content: &str,
    attach: Option<&[Alternative]>,
) -> Result<(), ()> {
    let mut c = smtp.upstream.as_mut().unwrap();
    let buf = &mut smtp.buf;

    c.write(format!("MAIL FROM: <{}>\r\n", from).as_bytes())
        .await
        .unwrap();
    c.read(buf).await.unwrap();

    c.write(format!("RCPT TO: <{}>\r\n", to).as_bytes())
        .await
        .unwrap();
    c.read(buf).await.unwrap();

    c.write(b"DATA\r\n").await.unwrap();
    loop {
        match c.read(buf).await {
            Ok(n) if n == 0 => {
                break;
            }
            Ok(n) => {
                if buf[..n].ends_with(b"<CR><LF>.<CR><LF>\r\n") {
                    break;
                } else if buf[..n].ends_with(b"<CR><LF>.<CR><LF>.\r\n") {
                    break;
                }
                println!("{}", String::from_utf8_lossy(&buf[..n]));
            }
            Err(e) => {
                eprintln!("{}", e);
                return Err(());
            }
        }
    }

    match mime_encode(
        from,
        to,
        subject,
        content_transfer_encoding,
        content_type,
        content,
        attach,
    )
    .await
    {
        Ok(encoded) => {
            c.write_all(&encoded).await.unwrap();
        }
        Err(_) => {
            smtp.upstream = None;
            return Err(());
        }
    }

    c.write(b"\r\n.\r\n").await.unwrap();
    let n = c.read(buf).await.unwrap();
    print!("{}", String::from_utf8_lossy(&buf[0..n]));

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn smtp_test() {
        let mut smtp = SmtpBuilder::new()
            .email("root@jckeep.top")
            // .token("fhbwysogaphydiga")
            .host("smtp.jckeep.top:25")
            .build()
            .await;

        let a = Alternative {
            filename: None,
            content: HTML.to_string(),
            content_type: ContentType::TextHtml,
            encoding: ContentTransferEncoding::QuotedPrintable,
        };
        let b = Alternative {
            filename: Some(String::from("传输层.pdf")),
            content: "".to_string(),
            content_type: ContentType::ApplicationPdf,
            encoding: ContentTransferEncoding::Base64,
        };
        let c = Alternative {
            filename: Some(String::from("头像.jpeg")),
            content: "".to_string(),
            content_type: ContentType::ImageJpeg,
            encoding: ContentTransferEncoding::Bit7,
        };
        let d = Alternative {
            filename: Some(String::from("src/main.rs")),
            content: "".to_string(),
            content_type: ContentType::TextPlain,
            encoding: ContentTransferEncoding::Bit7,
        };
        let v = vec![a, c, d];

        let mut buf = bytes::BytesMut::new();
        buf.put_slice("a".as_bytes());

        smtp.send(
            "root@jckeep.top",
            "2407018371@qq.com",
            "Nginx",
            ContentTransferEncoding::Bit7,
            ContentType::MultipartMixed,
            HTML,
            Some(&v),
        )
        .await;
    }
}

const HTML: &str = r#"
<!DOCTYPE html>
<html>
<head>
<title>Welcome to nginx!</title>
<style>
    body {
        width: 35em;
        margin: 0 auto;
        font-family: Tahoma, Verdana, Arial, sans-serif;
    }
</style>
</head>
<body>
<h1>Welcome to nginx!</h1>
<p>If you see this page, the nginx web server is successfully installed and
working. Further configuration is required.</p>

<p>For online documentation and support please refer to
<a href="http://nginx.org/">nginx.org</a>.<br/>
Commercial support is available at
<a href="http://nginx.com/">nginx.com</a>.</p>

<p><em>Thank you for using nginx.</em></p>
</body>
</html>
.
"#;
