use anyhow::Result;
use cap_std::fs::File as CapStdFile;
use std::collections::HashMap;
use std::io::prelude::*;
use wasmtime::*;
use wasmtime_wasi::*;

pub fn old_wasm_loader(module: String) -> Result<String> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Instantiating a stdout file for later usage
    let temp_file = tempfile::tempfile()?;
    let mut capstd_file = CapStdFile::from_std(temp_file);
    let wasmtime_file = wasmtime_wasi::file::File::from_cap_std(capstd_file.try_clone()?);

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .stdout(Box::new(wasmtime_file))
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, module)?;
    linker.module(&mut store, "", &module)?;
    let res = linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let instance = linker.instantiate(&mut store, &module)?;
    let instance_main = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    instance_main.call(&mut store, ())?;

    // Let's try to read the thing
    let mut output = String::new();
    capstd_file.seek(std::io::SeekFrom::Start(0))?; // seek to the beginning
    let stdout_file_handle = CapStdFile::into_std(capstd_file);
    let mut buf_reader = std::io::BufReader::new(stdout_file_handle);
    buf_reader.read_to_string(&mut output)?;

    Ok(output)
}

pub fn wasm_loader(module: String, parameters: HashMap<String, String>) -> Result<String> {
    // Define the WASI functions globally on the `Config`.
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)?;

    // Instantiating a stdout file for later usage
    let temp_file = tempfile::tempfile()?;
    let mut capstd_file = CapStdFile::from_std(temp_file);
    let wasmtime_file = wasmtime_wasi::file::File::from_cap_std(capstd_file.try_clone()?);

    let envs: Vec<(String, String)> = parameters
        .iter()
        .map(|(name, value)| (name.clone(), value.clone()))
        .collect();

    // Create a WASI context and put it in a Store; all instances in the store
    // share this context. `WasiCtxBuilder` provides a number of ways to
    // configure what the target program will have access to.
    let wasi = WasiCtxBuilder::new()
        .stdout(Box::new(wasmtime_file))
        .envs(&envs)?
        .build();
    let mut store = Store::new(&engine, wasi);

    // Instantiate our module with the imports we've created, and run it.
    let module = Module::from_file(&engine, module)?;
    linker.module(&mut store, "", &module)?;
    let res = linker
        .get_default(&mut store, "")?
        .typed::<(), ()>(&store)?
        .call(&mut store, ())?;

    let instance = linker.instantiate(&mut store, &module)?;
    let instance_main = instance.get_typed_func::<(), ()>(&mut store, "_start")?;
    instance_main.call(&mut store, ())?;

    // Let's try to read the thing
    let mut output = String::new();
    capstd_file.seek(std::io::SeekFrom::Start(0))?; // seek to the beginning
    let stdout_file_handle = CapStdFile::into_std(capstd_file);
    let mut buf_reader = std::io::BufReader::new(stdout_file_handle);
    buf_reader.read_to_string(&mut output)?;

    Ok(output)
}
