use std::fs;
use std::path::Path;

use anyhow::Result;

pub fn execute(path: &str) -> Result<()> {
    let src = Path::new(path);
    if !src.exists() {
        anyhow::bail!("model file not found: {path}");
    }

    fs::create_dir_all("models")?;
    let dest = Path::new("models").join(src.file_name().unwrap());
    fs::copy(src, &dest)?;

    println!("clankeRS add-model: copied to {}", dest.display());
    println!(
        "Add to clankeRS.toml:\n\n[model.detector]\npath = \"{}\"",
        dest.display()
    );
    Ok(())
}
