# Miska

**Miska** is a simple attempt at creating a lambda bucket server, which can
execute `WebAssembly` lambdas on an `Actix Web` server through the use of
`wasmtime`.

## Running the server

Clone the repository, open its folder in the terminal and run

```sh
cargo run
```

Then open the browser and go to

```
https://8080/
```

to see the homepage and to

```
https://8080/lambda-name
```

to start the lambda of name `lambda-name`
