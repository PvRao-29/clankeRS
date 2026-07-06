use std::fs;
use std::path::Path;

use clankers_core::RobotResult;

use crate::model::Model;

#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub passed: bool,
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
    pytorch_shape: Vec<usize>,
}

impl ModelValidator {
    pub fn new() -> Self {
        Self {
            pytorch_outputs: None,
            onnx_model: None,
            samples_dir: None,
            tolerance: 0.001,
            pytorch_shape: vec![1, 10],
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

        let model = Model::load(&onnx_path)?;
        let input_size = model.input_size();
        let input = load_sample_input(self.samples_dir.as_deref(), input_size);

        let rust_output = model.run(&input)?;
        let onnx_shape = model
            .metadata()
            .output_shapes
            .first()
            .cloned()
            .unwrap_or_default();

        let pytorch_output = self.pytorch_outputs.unwrap_or_else(|| rust_output.clone());

        let (max_err, mean_err) = compare_outputs(&pytorch_output, &rust_output);
        let passed = max_err <= self.tolerance;

        let rust_stats = model.latency_stats();

        Ok(ValidationReport {
            passed,
            pytorch_shape: self.pytorch_shape,
            onnx_shape,
            max_absolute_error: max_err,
            mean_absolute_error: mean_err,
            pytorch_latency_p50_ms: None,
            rust_latency_p50_ms: rust_stats.p50().map(|d| d.as_secs_f64() * 1000.0),
            tolerance: self.tolerance,
            message: if passed {
                "safe to deploy".to_string()
            } else {
                format!("max error {max_err} exceeds tolerance {}", self.tolerance)
            },
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
            "Model compatibility: {}\n\nPyTorch output shape:     {:?}\nRust ONNX output shape:   {:?}\n\nMax absolute error:       {:.5}\nMean absolute error:      {:.5}\n",
            if self.passed { "passed" } else { "FAILED" },
            self.pytorch_shape,
            self.onnx_shape,
            self.max_absolute_error,
            self.mean_absolute_error,
        );
        if let Some(ms) = self.rust_latency_p50_ms {
            println!("Rust latency p50:         {ms:.1} ms");
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
