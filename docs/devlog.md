# Devlog

Here I will be tracking how I set up and develop the project.

Steps:
- installing Actix Web

## Installing Actix Web

[https://actix.rs/docs/getting-started]

After initializing the project with `cargo new`, I added Actix Web in the
dependencies in `cargo.toml`:

```toml
[dependencies]
actix-web = "4"
```

then copied and pasted the demo code from the Actix docs into `src/main.rs`:

```rust
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

## Installing wasmtime

```toml
[dependencies]
wasmtime = "7"
wasmtime-wasi = "7.0.0"
```
I added both wasmtime (a WASM engine for Rust) and its companion crate
`wasmtime-wasi`, which adds the *WebAssembly System Interface* to `wasmtime`.

The function
```rust
#[get("/{module}")]
async fn handler(module: Path<String>) -> impl Responder {
    let wasm_module = format!("{}{}", module, ".wasm");
    let value = wasm_loader(wasm_module).expect("Module not loaded");
    HttpResponse::Ok().body(value)
}
```
creates a handler which calls upon the `wasm_loader` function located in
`lib.rs` which can load WASM modules through `wasmtime`.

## Figuring out how to use Wasmtime

This is not particularly easy to figure out, because the documentation is, as
it is almost always the case, based on unexplained examples.

This is the code that the `wasmtime` docs propose as an example for WASI in
`wasmtime` for `Rust`:

```rust
//! Example of instantiating a wasm module which uses WASI imports.

/*
You can execute this example with:
    cmake example/
    cargo run --example wasi
*/

use anyhow::Result;
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

fn main() -> Result<()> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .inherit_args()?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, "target/wasm32-wasi/debug/wasi.wasm")?;
    linker.module(&mut store, "", &module)?;
    linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    Ok(())
}
```

It's hard to understand what's actually going on. From some examples I found
online *(outside of the documentation, obviously)*, it seems like `Arc<T>`
pointers are needed to access the results of a function from the `Store`
variable.
