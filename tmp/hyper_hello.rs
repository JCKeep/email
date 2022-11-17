#![allow(unused_imports)]
use std::net::SocketAddr;

use futures::StreamExt;
use hyper::{
    body,
    service::{make_service_fn, service_fn},
    Body, Method, Request, Response, Server, StatusCode,
};

async fn handle(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        (&Method::GET, "/auth") => {
            // read body
            // let b = body::to_bytes(req.body_mut()).await.unwrap();
            // let s = b.as_ref();
            // println!("body: {:?}", s);

            // HeaderValue::from_static("application/json")
            // response.headers_mut().insert(
            //     hyper::header::CONTENT_TYPE,
            //     "application/json".parse().unwrap(),
            // );
            // *response.body_mut() =
            //     Body::from("{\"username\": \"jckeep\", \"password\": \"123456\"}");
            // Body::from(bytes::Bytes::from_static(b"{\"username\": \"jckeep\", \"password\": \"123456\"}"));
            *response.status_mut() = StatusCode::OK;
        }
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            *response.body_mut() = Body::from("Not Found");
        }
    }

    Ok(response)
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8088));

    let make_svr = make_service_fn(|_conn| async { Ok::<_, hyper::Error>(service_fn(handle)) });

    let server = Server::bind(&addr).serve(make_svr);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
