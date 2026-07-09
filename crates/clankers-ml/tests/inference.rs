//! Integration tests for model loading, inference, and validation.
//!
//! The inference tests need the `onnxruntime` feature (on by default) and the
//! sample models under `sample_data/models/`. They are resolved relative to the
//! crate manifest so they work regardless of the test runner's CWD.

use clankers_ml::Model;

#[cfg(feature = "onnxruntime")]
fn sample_model(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../sample_data/models")
        .join(name)
}

#[test]
fn builder_requires_path() {
    // No ONNX runtime needed: the missing-path check happens before loading.
    // `Model` is not `Debug`, so match instead of `unwrap_err`.
    let err = match Model::builder().build() {
        Ok(_) => panic!("expected an error when no path is set"),
        Err(e) => e,
    };
    assert!(err.to_string().contains("model path not set"));
}

#[cfg(not(feature = "onnxruntime"))]
#[test]
fn load_without_backend_feature_errors() {
    use clankers_ml::backend::load_onnx;
    assert!(load_onnx("whatever.onnx").is_err());
}

#[cfg(feature = "onnxruntime")]
mod onnx {
    use super::*;
    use clankers_ml::ModelValidator;

    #[test]
    fn load_and_run_detector() {
        let mut model = Model::load(sample_model("detector.onnx")).unwrap();
        assert_eq!(model.metadata().backend, "onnxruntime");

        let size = model.input_size();
        assert!(size > 0, "input size should be known from model metadata");

        let output = model.run(&vec![0.5f32; size]).unwrap();
        assert!(!output.is_empty());
        assert_eq!(model.latency_stats().count(), 1);
    }

    #[test]
    fn validate_detector_against_reference() {
        let samples = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../sample_data/detector_inputs");
        let report = ModelValidator::new()
            .onnx_model(sample_model("detector.onnx").to_str().unwrap())
            .sample_inputs(samples.to_str().unwrap())
            .validate()
            .unwrap();

        assert!(report.has_reference, "expected_output.json should be found");
        assert!(
            report.passed,
            "detector should match its committed reference: {}",
            report.message
        );
    }

    #[test]
    fn validate_without_reference_does_not_pass() {
        let report = ModelValidator::new()
            .onnx_model(sample_model("policy.onnx").to_str().unwrap())
            .validate()
            .unwrap();
        // No samples dir / reference -> must not fabricate a pass.
        assert!(!report.has_reference);
        assert!(!report.passed);
    }

    #[test]
    fn validate_flags_length_mismatch() {
        let report = ModelValidator::new()
            .onnx_model(sample_model("detector.onnx").to_str().unwrap())
            .pytorch_outputs(vec![0.0f32; 9999])
            .validate()
            .unwrap();
        assert!(!report.passed);
        assert!(report.message.contains("length mismatch"));
    }
}
