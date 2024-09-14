extern crate bindgen;
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=path/to/Cargo.lock");
    // Compile the C code
    cc::Build::new().file("src/gpc.c").compile("gpc");

    // Generate Rust bindings from the C header file
    let bindings = bindgen::Builder::default()
        .header("src/gpc.h")
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the `src/gpc.rs` file
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("gpc.rs"))
        .expect("Couldn't write bindings!");
}
