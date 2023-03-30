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

### The easier way

Since I don't understand how to use `Arc` pointers and all of that, I found
an easier way (totally unorthodox, of course) to get the string result through
a temporary file.

On top of the usual stuff I had from before, I opened a temporary file, to
convert it into a `cap-std` file (I added `cap-std` and `tempfile` to the
dependencies), and finally into a `wasmtime-wasi` file.

```rust
// Instantiating a stdout file for later usage
let temp_file = tempfile::tempfile()?;
let mut capstd_file = CapStdFile::from_std(temp_file);
let wasmtime_file = wasmtime_wasi::file::File::from_cap_std(capstd_file.try_clone()?);
```

The `wasmtime-wasi` file possesses the correct traits to be passed upon to
`WasiCtxBuilder`:

```rust
// Create a WASI context and put it in a Store; all instances in the store
// share this context. `WasiCtxBuilder` provides a number of ways to
// configure what the target program will have access to.
let wasi = WasiCtxBuilder::new()
    .stdout(Box::new(wasmtime_file))
    .build();
let mut store = Store::new(&engine, wasi);
```

In the end, we can retrieve the result from memory and copy it into
a string, by rewinding the file and then using a BufReader:

```rust
// Let's try to read the thing
let mut output = String::new();
capstd_file.seek(std::io::SeekFrom::Start(0))?; // seek to the beginning
let stdout_file_handle = CapStdFile::into_std(capstd_file);
let mut buf_reader = std::io::BufReader::new(stdout_file_handle);
buf_reader.read_to_string(&mut output)?;

Ok(output)
```

Also, I found out that the current version of `AssemblyScript` doesn't support
`WASI` out of the box anymore. I found a package called `wasi-shim`, that,
when added as a dev dependency to the `AssemblyScript` project, will handle
`WASI` correctly.

### Passing data *to* the WASM module

After finding a way to retrieve the result as a string, the next step is
figuring out a way to pass data to the WASM function.

This seems way easier than retrieving the result. It is possible to pass a
reference to a vector of name/value tuples of environment variables to
`WasiCtxBuilder`:

```rust
let envs: Vec<(String, String)> = vec![...];

let wasi = WasiCtxBuilder::new()
    .stdout(Box::new(stdout))
    .envs(&envs)?
    .build();
```

The updated handler, using the `Actix` way of handling queries, will look
like this:

```rust
#[get("/{module}")]
async fn handler(module: Path<String>, query: Query<HashMap<String, String>>)
    -> impl Responder {
    let wasm_module = format!("{}{}", module, ".wasm");
    let value = invoke_wasm_module(wasm_module, query.into_inner())
      .expect("Module not loaded");
    HttpResponse::Ok().body(value)
}
```

## Fixing the double return

Calling `stringtest.wasm` used to, for some reason I couldn't figure out,
return the *Hello, World!* string twice. After adding some tests, I found out
why it was the case:

```rust
let res = linker
    .get_default(&mut store, "")?
    .typed::<(), ()>(&store)?
    .call(&mut store, ())?;
```

I had this line of dead code laying around in both the `wasm_loader` and
`wasm_loader_old` functions, and it was basically calling the `_start`
function before its time. It was called twice on every run, thus the double
result. By removing it, now `stringtest.wasm` only returns *Hello, World!"
once.
