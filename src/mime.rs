#![allow(unused)]

use base64::encode;
use bytes::BufMut;
use encoding::{EncoderTrap, Encoding};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Clone, Copy)]
pub enum ContentTransferEncoding {
    Base64,
    Bit7,
    QuotedPrintable,
}

#[derive(Debug, Clone, Copy)]
pub enum ContentType {
    TextHtml,
    TextPlain,
    MultipartMixed,
    MultipartAlternative,
    ImageJpeg,
    ImageGif,
    ImagePng,
    ApplicationPdf,
    ApplicationZip,
    ApplicationRar,
    VideoMp4,
    ApplicationPPTX,
    ApplicationWORD,
    ApplicationEXCEL,
    ApplicationOctetStream,
}

#[derive(Debug, Clone, Copy)]
pub enum ContentDisposition {
    Attachment,
    Inline,
}

#[derive(Debug, Clone, Copy)]
pub enum CharSet {
    Utf8,
}

impl ContentTransferEncoding {
    pub const VALUE_MAP: [&'static str; 3] = ["base64", "7bit", "quoted-printable"];
}

impl ContentType {
    pub const VALUE_MAP: [&'static str; 15] = [
        "text/html",
        "text/plain",
        "multipart/mixed",
        "multipart/alternative",
        "image/jpeg",
        "image/gif",
        "image/png",
        "application/pdf",
        "application/zip",
        "application/rar",
        "video/mp4",
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "application/octet-stream",
    ];
}

pub fn parse_content_type(s: &str) -> ContentType {
    if s.ends_with(".jpeg") || s.ends_with(".jpg") {
        ContentType::ImageJpeg
    } else if s.ends_with(".mp4") || s.ends_with(".m4a") {
        ContentType::VideoMp4
    } else if s.ends_with(".gif") {
        ContentType::ImageGif
    } else if s.ends_with(".png") {
        ContentType::ImagePng
    } else if s.ends_with(".pdf") {
        ContentType::ApplicationPdf
    } else if s.ends_with(".rar") {
        ContentType::ApplicationRar
    } else if s.ends_with(".zip") {
        ContentType::ApplicationZip
    } else if s.ends_with(".docx") {
        ContentType::ApplicationWORD
    } else if s.ends_with(".pptx") {
        ContentType::ApplicationPPTX
    } else if s.ends_with(".xls") {
        ContentType::ApplicationEXCEL
    } else if s.ends_with(".c")
        || s.ends_with(".rs")
        || s.ends_with(".cpp")
        || s.ends_with(".h")
        || s.ends_with(".txt")
        || s.ends_with(".toml")
    {
        ContentType::TextPlain
    } else {
        ContentType::ApplicationOctetStream
    }
}

#[derive(Debug)]
pub struct Alternative {
    pub filename: Option<String>,
    pub content: String,
    pub content_type: ContentType,
    pub encoding: ContentTransferEncoding,
}

pub async fn mime_encode(
    from: &str,
    to: &str,
    subject: &str,
    content_transfer_encoding: ContentTransferEncoding,
    content_type: ContentType,
    content: &str,
    attach: Option<&[Alternative]>,
) -> Result<Vec<u8>, ()> {
    let mut encoded = Vec::new();

    let ec = ContentTransferEncoding::VALUE_MAP[content_transfer_encoding as usize];
    let ct = ContentType::VALUE_MAP[content_type as usize];

    for b in format!(
        "From: {}\r\nTo: {}\r\nSubject: {}\r\nMIME-Version: 1.0\r\n",
        from, to, subject
    )
    .as_bytes()
    {
        encoded.put_u8(*b);
    }

    let multipart = match content_type {
        ContentType::MultipartMixed | ContentType::MultipartAlternative => {
            for b in format!("Content-Type: {}; boundary=\"0123456789\"\r\n", ct).as_bytes() {
                encoded.put_u8(*b);
            }

            true
        }
        _ => {
            for b in format!("Content-Type: {}; charset=\"utf-8\"\r\n", ct).as_bytes() {
                encoded.put_u8(*b);
            }

            false
        }
    };

    if multipart {
        if attach.is_none() {
            eprintln!("attach not found");
            return Err(());
        }
        for elts in attach.unwrap() {
            let ec = ContentTransferEncoding::VALUE_MAP[elts.encoding as usize];
            let ct = ContentType::VALUE_MAP[elts.content_type as usize];
            if let Some(ref filename) = elts.filename {
                for b in format!("\r\n--0123456789\r\nContent-Type: {}; name=\"{}\"; charset=\"utf-8\"\r\nContent-Transfer-Encoding: base64\r\n\r\n", ct, filename).as_bytes() {
                    encoded.put_u8(*b);
                }
                match File::open(filename).await {
                    Ok(mut file) => {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).await.unwrap();
                        for b in encode(&buf).as_bytes() {
                            encoded.put_u8(*b);
                        }
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(());
                    }
                }
            } else {
                for b in format!("\r\n--0123456789\r\nContent-Type: {}; charset=\"utf-8\"\r\nContent-Transfer-Encoding: {}\r\n\r\n", ct, ec).as_bytes() {
                    encoded.put_u8(*b);
                }
                match elts.encoding {
                    ContentTransferEncoding::Base64 => {
                        for b in encode(&elts.content).as_bytes() {
                            encoded.put_u8(*b);
                        }
                    }
                    ContentTransferEncoding::Bit7 => {
                        match encoding::all::ASCII.encode(&elts.content, EncoderTrap::Strict) {
                            Ok(mut v) => {
                                encoded.append(&mut v);
                            }
                            Err(e) => {
                                eprintln!("{}", e);
                                return Err(());
                            }
                        }
                    }
                    ContentTransferEncoding::QuotedPrintable => {
                        encoded.append(&mut quoted_printable::encode(&elts.content));
                    }
                }
            }
        }
        for b in b"\r\n--0123456789--\r\n" {
            encoded.put_u8(*b);
        }
    } else {
        match content_transfer_encoding {
            ContentTransferEncoding::Base64 => {
                for b in format!("Content-Transfer-Encoding: {}\r\n\r\n", ec).as_bytes() {
                    encoded.put_u8(*b);
                }
                for b in encode(content).as_bytes() {
                    encoded.put_u8(*b);
                }
            }
            ContentTransferEncoding::Bit7 => {
                for b in format!("Content-Transfer-Encoding: {}\r\n\r\n", ec).as_bytes() {
                    encoded.put_u8(*b);
                }
                match encoding::all::ASCII.encode(content, EncoderTrap::Strict) {
                    Ok(mut v) => encoded.append(&mut v),
                    Err(e) => {
                        eprintln!("{}", e);
                        return Err(());
                    }
                }
            }
            ContentTransferEncoding::QuotedPrintable => {
                for b in format!("Content-Transfer-Encoding: {}\r\n\r\n", ec).as_bytes() {
                    encoded.put_u8(*b);
                }
                encoded.append(&mut quoted_printable::encode(content));
            }
        }
    }

    Ok(encoded)
}

pub async fn mime_decode(content: &str) {}
