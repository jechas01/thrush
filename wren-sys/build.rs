extern crate bindgen;

use std::env;
use std::path::PathBuf;

use std::process::Command;

fn main() {
    let top_dir = env::var("CARGO_MANIFEST_DIR").expect("manifest directory");
    assert!(
        Command::new("make")
            .current_dir(format!("{}/wren", top_dir))
            .args(&["vm"])
            .status()
            .expect("failed to run 'make'")
            .success()
    );
    println!("cargo:rustc-link-search=native={}/wren/lib", top_dir);
    println!("cargo:rustc-link-lib=static=wren");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/wren/src/include/wren.h", top_dir))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
