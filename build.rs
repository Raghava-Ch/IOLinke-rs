fn main() {
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:warning=C header generation disabled. Install cbindgen and enable in Cargo.toml to generate headers.");
}
