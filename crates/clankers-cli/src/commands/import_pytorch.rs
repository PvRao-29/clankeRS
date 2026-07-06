use std::process::Command;

use anyhow::{Context, Result};

pub fn execute(model: &str, checkpoint: &str, output: &str, opset: Option<u32>) -> Result<()> {
    let script = find_script("export_pytorch_to_onnx.py")?;
    let mut cmd = Command::new("python3");
    cmd.arg(&script)
        .arg("--model")
        .arg(model)
        .arg("--checkpoint")
        .arg(checkpoint)
        .arg("--output")
        .arg(output);

    if let Some(opset) = opset {
        cmd.arg("--opset").arg(opset.to_string());
    }

    let status = cmd.status().context("failed to run export script")?;
    if !status.success() {
        anyhow::bail!("PyTorch export failed");
    }

    println!("Exported ONNX model to {output}");
    Ok(())
}

fn find_script(name: &str) -> Result<std::path::PathBuf> {
    let candidates = [
        format!("scripts/{name}"),
        format!("../scripts/{name}"),
        format!("../../scripts/{name}"),
    ];
    for c in candidates {
        let p = std::path::Path::new(&c);
        if p.exists() {
            return Ok(p.to_path_buf());
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        for ancestor in cwd.ancestors() {
            let p = ancestor.join("scripts").join(name);
            if p.exists() {
                return Ok(p);
            }
        }
    }
    anyhow::bail!("script {name} not found")
}
