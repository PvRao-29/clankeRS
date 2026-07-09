//! Smoke test for the `clankers bench` command against a sample ONNX model.

fn sample_model() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../sample_data/models/detector.onnx")
}

#[test]
fn bench_reports_percentiles_for_sample_model() {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_clankers"))
        .args([
            "bench",
            "--model",
            sample_model().to_str().unwrap(),
            "--warmup",
            "2",
            "--iters",
            "10",
        ])
        .output()
        .expect("run clankers bench");

    assert!(
        output.status.success(),
        "bench failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("clankeRS bench — onnxruntime"), "{stdout}");
    assert!(stdout.contains("latency p50"), "{stdout}");
    assert!(stdout.contains("latency p99"), "{stdout}");
    assert!(stdout.contains("conversion copies"), "{stdout}");
}
