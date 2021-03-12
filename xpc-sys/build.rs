extern crate bindgen;
extern crate xcrun;

use std::env;
use std::path::PathBuf;

use xcrun::SDK;

static MACOS_INCLUDE_PATH: &str = "/usr/include";

fn main() {
    let sdk_path = xcrun::find_sdk(SDK::macOS(None))
        .and_then(|pb| pb.to_str().map(String::from))
        .and_then(|p| p.strip_suffix("\n").map(String::from))
        .expect("macOS SDK Required");

    let xpc_path = format!("{}{}/xpc/xpc.h", sdk_path, MACOS_INCLUDE_PATH);
    let bootstrap_path = format!("{}{}/bootstrap.h", sdk_path, MACOS_INCLUDE_PATH);
    let sys_types = format!("{}{}/sys/types.h", sdk_path, MACOS_INCLUDE_PATH);
    let sysctl = format!("{}{}/sys/sysctl.h", sdk_path, MACOS_INCLUDE_PATH);

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(xpc_path)
        .header(bootstrap_path)
        .header(sys_types)
        .header(sysctl)
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
