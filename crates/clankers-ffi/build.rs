use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let config = crate_dir.join("cbindgen.toml");
    let header_path = crate_dir
        .join("..")
        .join("..")
        .join("cpp")
        .join("include")
        .join("clankers")
        .join("clankers.h");

    if let Some(parent) = header_path.parent() {
        std::fs::create_dir_all(parent).expect("create header directory");
    }

    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file(&config).expect("load cbindgen.toml"))
        .generate()
        .expect("generate bindings");

    let mut body = Vec::new();
    bindings.write(&mut body);
    let body = String::from_utf8(body).expect("header is utf-8");

    let header = format!("/* clankeRS C ABI — see cpp/README.md for C++ wrappers. */\n{body}");
    std::fs::write(&header_path, header).expect("write clankers.h");

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
