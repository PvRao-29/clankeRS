use std::env;
use std::path::{Path, PathBuf};

fn write_header(path: &Path, header: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create header directory");
    }
    std::fs::write(path, header).expect("write clankers.h");
}

fn main() {
    let crate_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let config = crate_dir.join("cbindgen.toml");

    let bindings = cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(cbindgen::Config::from_file(&config).expect("load cbindgen.toml"))
        .generate()
        .expect("generate bindings");

    let mut body = Vec::new();
    bindings.write(&mut body);
    let body = String::from_utf8(body).expect("header is utf-8");

    let header = format!("/* clankeRS C ABI — see cpp/README.md for C++ wrappers. */\n{body}");

    // Always emit inside the published crate (crates.io consumers build from here).
    let in_crate = crate_dir
        .join("include")
        .join("clankers")
        .join("clankers.h");
    write_header(&in_crate, &header);

    // When building inside the clankeRS repo, keep cpp/ in sync for the CMake SDK.
    let cpp_header = crate_dir
        .join("..")
        .join("..")
        .join("cpp")
        .join("include")
        .join("clankers")
        .join("clankers.h");
    if cpp_header
        .parent()
        .and_then(|dir| dir.parent())
        .map(|dir| dir.join("clankers.hpp").is_file())
        .unwrap_or(false)
    {
        write_header(&cpp_header, &header);
    }

    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=cbindgen.toml");
}
