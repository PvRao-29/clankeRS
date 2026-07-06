use std::process::Command;

use anyhow::{Context, Result};

pub fn execute(name: &str) -> Result<()> {
    match name {
        "camera-perception" | "camera_perception" => {
            println!("clankeRS demo: camera_perception_node\n");
            let status = Command::new("cargo")
                .args(["run", "-p", "camera_perception_node"])
                .current_dir(find_workspace_root()?)
                .status()
                .context("run camera perception demo")?;
            if !status.success() {
                anyhow::bail!("demo failed");
            }
        }
        other => anyhow::bail!("unknown demo: {other}"),
    }
    Ok(())
}

fn find_workspace_root() -> Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;
    for ancestor in cwd.ancestors() {
        if ancestor.join("Cargo.toml").exists() && ancestor.join("crates").exists() {
            return Ok(ancestor.to_path_buf());
        }
    }
    Ok(cwd)
}
