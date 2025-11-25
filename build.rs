use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed=o1heap/o1heap/o1heap.c");
    println!("cargo:rerun-if-changed=o1heap/o1heap/o1heap.h");

    // Compile o1heap C library
    cc::Build::new()
        .file("o1heap/o1heap/o1heap.c")
        // Disable assertions for embedded targets (no __assert_func)
        .define("O1HEAP_ASSERT(x)", "(void)(x)")
        .compile("o1heap");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("o1heap/o1heap/o1heap.h")
        .use_core()
        .clang_arg("-Io1heap/o1heap")
        // Only generate bindings for o1heap functions
        .allowlist_function("o1heap.*")
        .allowlist_type("O1Heap.*")
        .allowlist_var("o1heap.*")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldnt write bindings!");

    println!("cargo:rustc-link-lib=static=o1heap");
}
