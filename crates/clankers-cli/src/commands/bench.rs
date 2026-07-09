//! `clankers bench` — measure inference latency and copy/allocation behaviour.
//!
//! Loads a model into the modular [`InferenceEngine`], warms it up, then times a
//! run loop and reports load time, first-inference latency, p50/p95/p99/max, and
//! the per-run copy / allocation accounting the engine tracks. `--backend noop`
//! swaps in an identity backend over the same input shapes as a pure-memcpy
//! baseline to compare against the real ONNX Runtime path.

use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};

use clankers_ml::backend::{BackendSession, TensorSpec};
use clankers_ml::inference::{noop_engine_from_config, onnx_engine_from_config};
use clankers_ml::inference::InferenceEngine;
use clankers_ml::{InferenceStats};
use clankers_tensor::{Tensor, TensorView};
use clankers_core::config::ModelConfig;

pub fn execute(
    model: &str,
    backend: &str,
    input: Option<&str>,
    warmup: u32,
    iters: u32,
) -> Result<()> {
    if iters == 0 {
        bail!("--iters must be at least 1");
    }

    match backend {
        "onnxruntime" | "onnx" => bench_onnx(model, input, warmup, iters),
        "noop" => bench_noop(model, input, warmup, iters),
        other => bail!("unknown backend {other:?} (expected 'onnxruntime' or 'noop')"),
    }
}

fn bench_onnx(model: &str, input: Option<&str>, warmup: u32, iters: u32) -> Result<()> {
    let config = bench_model_config("onnxruntime", warmup);
    let load_start = Instant::now();
    let mut engine = onnx_engine_from_config(&config, model)
        .with_context(|| format!("loading ONNX model {model}"))?;
    let load_time = load_start.elapsed();

    let inputs = build_inputs(engine.input_specs(), input)?;
    let report = run_bench(&mut engine, &inputs, warmup, iters)?;
    report.print("onnxruntime", model, load_time);
    Ok(())
}

fn bench_noop(model: &str, input: Option<&str>, warmup: u32, iters: u32) -> Result<()> {
    let config = bench_model_config("noop", warmup);
    let load_start = Instant::now();
    let mut engine = noop_engine_from_config(&config, model)
        .with_context(|| format!("building noop engine for {model}"))?;
    let load_time = load_start.elapsed();

    let inputs = build_inputs(engine.input_specs(), input)?;
    let report = run_bench(&mut engine, &inputs, warmup, iters)?;
    report.print("noop", model, load_time);
    Ok(())
}

fn bench_model_config(backend: &str, warmup: u32) -> ModelConfig {
    ModelConfig {
        source_framework: None,
        path: String::new(),
        backend: backend.into(),
        device: "cpu".into(),
        warmup_runs: Some(warmup),
        max_latency_ms: None,
        input: None,
        output: None,
    }
}

/// Build one input tensor per spec: the first is loaded from `--input` (raw
/// little-endian `f32` bytes) when provided, the rest are zero-filled.
fn build_inputs(specs: &[TensorSpec], input: Option<&str>) -> Result<Vec<Tensor>> {
    if specs.is_empty() {
        bail!("model has no inputs to benchmark");
    }
    let mut tensors = Vec::with_capacity(specs.len());
    for (i, spec) in specs.iter().enumerate() {
        let shape = spec.shape.concrete_or_unit();
        if i == 0 {
            if let Some(path) = input {
                let bytes = std::fs::read(path).with_context(|| format!("reading {path}"))?;
                let expected = shape.num_bytes(spec.dtype.element_size());
                if bytes.len() != expected {
                    bail!(
                        "input file {path} has {} bytes but model input {:?} expects {expected}",
                        bytes.len(),
                        spec.name
                    );
                }
                tensors.push(Tensor::from_bytes(spec.dtype, shape, &bytes)?);
                continue;
            }
        }
        tensors.push(Tensor::zeros(spec.dtype, shape));
    }
    Ok(tensors)
}

struct BenchReport {
    iters: u32,
    first: Duration,
    p50: Duration,
    p95: Duration,
    p99: Duration,
    max: Duration,
    last: InferenceStats,
    arena_bytes: usize,
}

fn run_bench<S: BackendSession>(
    engine: &mut InferenceEngine<S>,
    inputs: &[Tensor],
    warmup: u32,
    iters: u32,
) -> Result<BenchReport> {
    let views: Vec<TensorView> = inputs.iter().map(|t| t.view()).collect();

    for _ in 0..warmup {
        engine
            .run_with_stats(&views)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
    }

    let (_out, first_stats) = engine
        .run_with_stats(&views)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut latencies = Vec::with_capacity(iters as usize);
    let mut last = InferenceStats::default();
    for _ in 0..iters {
        let (_out, stats) = engine
            .run_with_stats(&views)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        latencies.push(stats.latency);
        last = stats;
    }
    latencies.sort_unstable();

    Ok(BenchReport {
        iters,
        first: first_stats.latency,
        p50: percentile(&latencies, 50.0),
        p95: percentile(&latencies, 95.0),
        p99: percentile(&latencies, 99.0),
        max: latencies.last().copied().unwrap_or_default(),
        last,
        arena_bytes: engine.arena_bytes(),
    })
}

fn percentile(sorted: &[Duration], p: f64) -> Duration {
    if sorted.is_empty() {
        return Duration::ZERO;
    }
    let rank = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

impl BenchReport {
    fn print(&self, backend: &str, model: &str, load_time: Duration) {
        println!("clankeRS bench — {backend}");
        println!("  model:           {model}");
        println!("  load time:       {:.2} ms", ms(load_time));
        println!("  iterations:      {} (after warmup)", self.iters);
        println!("  first inference: {:.3} ms", ms(self.first));
        println!("  latency p50:     {:.3} ms", ms(self.p50));
        println!("  latency p95:     {:.3} ms", ms(self.p95));
        println!("  latency p99:     {:.3} ms", ms(self.p99));
        println!("  latency max:     {:.3} ms", ms(self.max));
        println!(
            "  per run:         {} conversion copies, {} allocations",
            self.last.clankers_copies, self.last.allocations
        );
        println!(
            "  backend copies:  {} ({} bytes)",
            self.last.backend.backend_copies, self.last.backend.backend_bytes_copied
        );
        println!("  arena bytes:     {}", self.arena_bytes);
    }
}
