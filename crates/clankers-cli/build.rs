use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_dir = Path::new(&manifest_dir);
    let src = manifest_dir.join("../../templates");
    let out = Path::new(&env::var("OUT_DIR").unwrap()).join("templates");

    if src.exists() {
        copy_dir_recursive(&src, &out).expect("copy templates into OUT_DIR");
        println!("cargo:rerun-if-changed={}", src.display());
    }

    println!("cargo:rustc-env=TEMPLATES_DIR={}", out.display());
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst)?;
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}
