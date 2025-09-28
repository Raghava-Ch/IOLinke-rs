use cbindgen;
use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let header_path = crate_dir.clone() + "/../target/include/iolinke_device.h";
    let config = cbindgen::Config::from_file(crate_dir.clone() + "/cbindgen.toml").unwrap();

    cbindgen::Builder::new()
        .with_crate(crate_dir.clone())
        .with_config(config)
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(header_path);
}
