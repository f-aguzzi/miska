# Devlog

Here I will be tracking how I set up and develop the project.

Steps:
- installing Actix Web

## Installing Actix Web

[https://actix.rs/docs/getting-started]

After initializing the project with `cargo new`, I added Actix Web in the
dependencies in `cargo.toml`:

```{toml}
[dependencies]
actix-web = "4"
```

then copied and pasted the demo code from the Actix docs into `src/main.rs`:

```{rust}
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```
This will open a port on 8080 and respond to HTTP requests with a simple
Hello World demo page.

The spawned `HttpServer' will use the provided handler functions to handle
different requests. I will use this to hande the lambda functions later
on.
