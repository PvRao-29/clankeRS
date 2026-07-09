use std::fs;
use std::path::Path;

use clankers_core::RobotResult;

use crate::model::Model;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub passed: bool,
    pub has_reference: bool,
    pub pytorch_shape: Vec<usize>,
    pub onnx_shape: Vec<usize>,
    pub max_absolute_error: f32,
    pub mean_absolute_error: f32,
    pub pytorch_latency_p50_ms: Option<f64>,
    pub rust_latency_p50_ms: Option<f64>,
    pub tolerance: f32,
    pub message: String,
}

pub struct ModelValidator {
    pytorch_outputs: Option<Vec<f32>>,
    onnx_model: Option<String>,
    samples_dir: Option<String>,
    tolerance: f32,
}

impl ModelValidator {
    pub fn new() -> Self {
        Self {
            pytorch_outputs: None,
            onnx_model: None,
            samples_dir: None,
            tolerance: 0.001,
        }
    }

    pub fn pytorch_reference(self, _path: impl Into<String>) -> Self {
        self
    }

    pub fn checkpoint(self, _path: impl Into<String>) -> Self {
        self
    }

    pub fn onnx_model(mut self, path: impl Into<String>) -> Self {
        self.onnx_model = Some(path.into());
        self
    }

    pub fn sample_inputs(mut self, dir: impl Into<String>) -> Self {
        self.samples_dir = Some(dir.into());
        self
    }

    /// Provide the reference (PyTorch) output directly, bypassing the
    /// `expected_output.json` lookup. Used by tests and by callers that run a
    /// live PyTorch reference themselves.
    pub fn pytorch_outputs(mut self, outputs: Vec<f32>) -> Self {
        self.pytorch_outputs = Some(outputs);
        self
    }

    pub fn tolerance(mut self, tol: f32) -> Self {
        self.tolerance = tol;
        self
    }

    pub fn validate(self) -> RobotResult<ValidationReport> {
        let onnx_path = self
            .onnx_model
            .ok_or_else(|| clankers_core::RobotError::Model("onnx model path required".into()))?;

        let mut model = Model::load(&onnx_path)?;
        let input_size = model.input_size();
        let input = load_sample_input(self.samples_dir.as_deref(), input_size);

        let rust_output = model.run(&input)?;
        let onnx_shape = model
            .metadata()
            .output_shapes
            .first()
            .cloned()
            .unwrap_or_default();
        let rust_stats = model.latency_stats();
        let rust_latency = rust_stats.p50().map(|d| d.as_secs_f64() * 1000.0);

        // Load the PyTorch reference output. Explicit outputs win; otherwise
        // read `expected_output.json` from the samples directory. If neither is
        // available we do NOT fabricate a pass — the whole point of this command
        // is to compare against an independent reference.
        let reference = self
            .pytorch_outputs
            .clone()
            .or_else(|| load_reference_output(self.samples_dir.as_deref()));

        let Some(reference) = reference else {
            return Ok(ValidationReport {
                passed: false,
                has_reference: false,
                pytorch_shape: Vec::new(),
                onnx_shape,
                max_absolute_error: f32::NAN,
                mean_absolute_error: f32::NAN,
                pytorch_latency_p50_ms: None,
                rust_latency_p50_ms: rust_latency,
                tolerance: self.tolerance,
                message: format!(
                    "no PyTorch reference found (expected {}/expected_output.json). \
                     Generate one with scripts/make_sample_models.py.",
                    self.samples_dir.as_deref().unwrap_or("<samples>")
                ),
            });
        };

        let pytorch_shape = if reference.len() == onnx_shape.iter().product::<usize>() {
            onnx_shape.clone()
        } else {
            vec![1, reference.len()]
        };

        let length_mismatch = reference.len() != rust_output.len();
        let (max_err, mean_err) = compare_outputs(&reference, &rust_output);
        let passed = !length_mismatch && max_err <= self.tolerance;

        let message = if length_mismatch {
            format!(
                "output length mismatch: PyTorch {} vs Rust {}",
                reference.len(),
                rust_output.len()
            )
        } else if passed {
            "safe to deploy".to_string()
        } else {
            format!(
                "max error {max_err:.6} exceeds tolerance {}",
                self.tolerance
            )
        };

        Ok(ValidationReport {
            passed,
            has_reference: true,
            pytorch_shape,
            onnx_shape,
            max_absolute_error: max_err,
            mean_absolute_error: mean_err,
            pytorch_latency_p50_ms: None,
            rust_latency_p50_ms: rust_latency,
            tolerance: self.tolerance,
            message,
        })
    }
}

impl Default for ModelValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationReport {
    pub fn print(&self) {
        println!(
            "Model compatibility: {}\n",
            if self.passed { "passed" } else { "FAILED" }
        );

        if !self.has_reference {
            println!("Status: {}", self.message);
            return;
        }

        println!(
            "PyTorch output shape:     {:?}\nRust ONNX output shape:   {:?}\n",
            self.pytorch_shape, self.onnx_shape,
        );
        println!(
            "Max absolute error:       {:.8}\nMean absolute error:      {:.8}\nTolerance:                {:.6}\n",
            self.max_absolute_error, self.mean_absolute_error, self.tolerance,
        );
        if let Some(ms) = self.pytorch_latency_p50_ms {
            println!("PyTorch latency p50:      {ms:.1} ms");
        }
        if let Some(ms) = self.rust_latency_p50_ms {
            println!("Rust latency p50:         {ms:.3} ms");
        }
        println!("\nStatus: {}", self.message);
    }
}

fn load_sample_input(samples_dir: Option<&str>, size: usize) -> Vec<f32> {
    if let Some(dir) = samples_dir {
        let path = Path::new(dir).join("input.json");
        if path.exists() {
            if let Ok(data) = fs::read_to_string(&path) {
                if let Ok(vals) = serde_json::from_str::<Vec<f32>>(&data) {
                    return vals;
                }
            }
        }
    }
    vec![0.5f32; size]
}

/// Load the PyTorch reference output written by `scripts/make_sample_models.py`.
fn load_reference_output(samples_dir: Option<&str>) -> Option<Vec<f32>> {
    let dir = samples_dir?;
    let path = Path::new(dir).join("expected_output.json");
    let data = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Vec<f32>>(&data).ok()
}

fn compare_outputs(a: &[f32], b: &[f32]) -> (f32, f32) {
    let n = a.len().min(b.len());
    if n == 0 {
        return (0.0, 0.0);
    }
    let mut max_err = 0.0f32;
    let mut sum_err = 0.0f32;
    for i in 0..n {
        let err = (a[i] - b[i]).abs();
        max_err = max_err.max(err);
        sum_err += err;
    }
    (max_err, sum_err / n as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_identical_is_zero() {
        let (max, mean) = compare_outputs(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
        assert_eq!(max, 0.0);
        assert_eq!(mean, 0.0);
    }

    #[test]
    fn compare_reports_max_and_mean() {
        let (max, mean) = compare_outputs(&[1.0, 2.0], &[1.5, 2.0]);
        assert!((max - 0.5).abs() < 1e-6);
        assert!((mean - 0.25).abs() < 1e-6);
    }
}
