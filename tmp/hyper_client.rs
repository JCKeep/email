#![allow(unused_imports)]
use std::net::SocketAddr;

use hyper::Client;

#[tokio::main]
async fn main() {
    //let addr = SocketAddr::from(([127, 0, 0, 1], 8088));

    let client = Client::new();
    let uri = "http://localhost:8088/user/1".parse().unwrap();
    let mut response = client.get(uri).await.unwrap();
    println!("{:#?}", response);
    let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
    let content = std::str::from_utf8(body.as_ref()).unwrap();
    println!("{}", content)
}
