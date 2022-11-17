use std::sync::Mutex;

use actix_web::{
    guard,
    web::{self, Path},
    App, HttpResponse, HttpServer, Responder,
};

struct Counter {
    counter: Mutex<i32>,
}

#[allow(unused_parens)]
async fn index(path: Path<(String)>, data: web::Data<Counter>) -> impl Responder {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    println!("username: {}", path);
    HttpResponse::Ok()
        .insert_header(("Content-Type", "application/json"))
        .insert_header(("Server", "Actix-Web"))
        .body("{\"code\": 200,\"msg\": \"success\", \"obj\": null}")
}

async fn count(data: web::Data<Counter>) -> impl Responder {
    let counter = data.counter.lock().unwrap();
    HttpResponse::Ok().body(format!("{}", counter))
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/counter").route(web::get().to(count)));
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let counter = web::Data::new(Counter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        App::new()
            .configure(config)
            .app_data(counter.clone())
            .service(
                web::scope("/test")
                    .guard(guard::fn_guard(|ctx| {
                        ctx.head().headers().contains_key("User-Agent")
                    }))
                    .route("/index/{username}", web::get().to(index)),
            )
    })
    .bind("127.0.0.1:8088")?
    .run()
    .await
}
