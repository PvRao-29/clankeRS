use std::process::Command;

use anyhow::{Context, Result};

pub fn execute() -> Result<()> {
    println!("clankeRS run");
    let status = Command::new("cargo")
        .args(["run", "--release"])
        .status()
        .context("failed to run cargo run")?;

    if !status.success() {
        anyhow::bail!("cargo run failed with status {status}");
    }
    Ok(())
}
