use actix_web::web::{Path, Query};
use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use miska::{old_wasm_loader, wasm_loader};
use std::collections::HashMap;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Miska!")
}

#[get("/about")]
async fn about() -> impl Responder {
    HttpResponse::Ok().body("This is the Miska lambda bucket.")
}

#[get("/stringtest")]
async fn stringtest() -> impl Responder {
    let wasm_module = format!("{}{}", "stringtest", ".wasm");
    let value = old_wasm_loader(wasm_module).expect("Module not loaded");
    HttpResponse::Ok().body(value)
}

#[get("/{module}")]
async fn handler(module: Path<String>, query: Query<HashMap<String, String>>) -> impl Responder {
    let wasm_module = format!("{}{}", module, ".wasm");
    let value = wasm_loader(wasm_module, query.into_inner()).expect("Module not loaded");
    HttpResponse::Ok().body(value)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(about)
            .service(stringtest)
            .service(handler)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
