use anyhow::Result;
use wasmtime::*;

pub fn wasm_loader(module_name: String) -> Result<()> {

    // Module loading and compilation
    println!("Compiling module...");
    let engine = Engine::default();
    let module = Module::from_file(&engine, module_name)?;

    // Creation of store and instance
    println!("Initializing...");
    let mut store = Store::new(&engine, ());    // Empty store for now
    let instance = Instance::new(&mut store, &module, &[])?;    // No imports for now

    // Locate the exported function
    let run = instance.get_typed_func::<(), i32>(&mut store, "run")?;

    // Call the function and print the result
    let res = run.call(&mut store, ())?;
    println!("WebAssembly result: {}", res);

    println!("Done.");
    Ok(())
}