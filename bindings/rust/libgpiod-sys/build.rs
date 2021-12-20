#[cfg(feature = "generate")]
extern crate bindgen;
#[cfg(feature = "generate")]
use std::env;
#[cfg(feature = "generate")]
use std::path::PathBuf;

#[cfg(feature = "generate")]
fn generate_bindings(files: &Vec<&str>) {
    // Tell cargo to invalidate the built crate whenever following files change
    println!("cargo:rerun-if-changed=wrapper.h");

    for file in files {
        println!("cargo:rerun-if-changed={}", file);
    }

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
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

fn build_gpiod(files: Vec<&str>) {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=../../../lib/");

    // Use the `cc` crate to build a C file and statically link it.
    cc::Build::new()
        .files(files)
        .define("_GNU_SOURCE", None)
        .define("GPIOD_VERSION_STR", "\"libgpio-sys\"")
        .include("../../../include")
        .compile("gpiod");
}

fn main() {
    let files = vec![
        "../../../lib/chip.c",
        "../../../lib/chip-info.c",
        "../../../lib/edge-event.c",
        "../../../lib/info-event.c",
        "../../../lib/internal.c",
        "../../../lib/line-config.c",
        "../../../lib/line-info.c",
        "../../../lib/line-request.c",
        "../../../lib/misc.c",
        "../../../lib/request-config.c",
    ];

    #[cfg(feature = "generate")]
    generate_bindings(&files);
    build_gpiod(files);
}
