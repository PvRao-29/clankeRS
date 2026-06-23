use anyhow::Result;
use clankers_ml::ModelValidator;

pub fn execute(
    _pytorch: Option<&str>,
    _checkpoint: Option<&str>,
    onnx: &str,
    samples: &str,
    tolerance: f32,
) -> Result<()> {
    let report = ModelValidator::new()
        .onnx_model(onnx)
        .sample_inputs(samples)
        .tolerance(tolerance)
        .validate()
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    report.print();

    if !report.passed {
        anyhow::bail!("model validation failed");
    }
    Ok(())
}
