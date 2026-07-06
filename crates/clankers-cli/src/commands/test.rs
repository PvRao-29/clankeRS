use std::process::Command;

use anyhow::{Context, Result};

pub async fn execute() -> Result<()> {
    println!("clankeRS test");
    let status = Command::new("cargo")
        .args(["test"])
        .status()
        .context("failed to run cargo test")?;

    if !status.success() {
        anyhow::bail!("cargo test failed");
    }
    Ok(())
}
