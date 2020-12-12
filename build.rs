extern crate bindgen;
extern crate xcrun;

use std::env;
use std::path::{Path, PathBuf};
use std::ffi::OsString;

static MACOS_INCLUDE_PATH: &str = "/usr/include";

fn main() {
    let sdk_pb = xcrun::find_sdk(xcrun::SDK::macOS(None))
        .expect("macOS SDK Required");

    let sdk_path = sdk_pb.to_str().unwrap().strip_suffix("\n").unwrap();
    let launch_path = format!("{}{}/launch.h", sdk_path, MACOS_INCLUDE_PATH);

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed={}", launch_path);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(launch_path)
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
