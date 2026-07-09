<p align="center">
  <strong>clankers-ml</strong><br>
  <em>Optimized ML inference and model deployment for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-ml"><img src="https://img.shields.io/static/v1?label=crates.io&message=v0.1.4&color=orange&style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-ml"><img src="https://docs.rs/clankers-ml/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-ml.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Load ONNX models through a pluggable backend runtime, bind zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html) inputs, and validate Rust output against offline PyTorch references.

[`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) is the main API. [`InferenceEngine`](https://docs.rs/clankers-ml/latest/clankers_ml/inference/struct.InferenceEngine.html) is the lower-level runtime underneath for custom backends and allocation policies.

## Install

```toml
clankers-ml = "0.1"
```

```bash
cargo add clankers-ml
```

**Note:** The default `onnxruntime` feature downloads the ONNX Runtime binary on first build (network required).

Disable ONNX for sim-only builds:

```toml
clankers-ml = { version = "0.1", default-features = false }
```

## Example

```rust
use clankers_ml::backend::OnnxRuntimeBackend;
use clankers_ml::{Model, ModelValidator};
use clankers_tensor::{DType, Layout, Shape, TensorView};

let mut model = Model::builder()
    .backend(OnnxRuntimeBackend::default())
    .load("models/detector.onnx")?;

let view = TensorView::from_f32(&input_f32, &Shape::from([1, 6]))?;
let outputs = model.run_named([("input", view)])?;

if let Some(stats) = model.stats() {
    assert_eq!(stats.clankers_copies, 0); // clankeRS avoided extra copies
}

let report = ModelValidator::new()
    .onnx_model("models/detector.onnx")
    .sample_inputs("fixtures/detector_inputs")
    .tolerance(0.001)
    .validate()?;
report.print();
```

Single-input convenience:

```rust
let mut model = Model::load("models/detector.onnx")?;
let output = model.run(&input_f32)?;
```

## Key types

| Type | Role |
|------|------|
| `Model` | Primary optimized inference runtime |
| `ModelBuilder` | Backend selection, warmup, latency limits, config bridge |
| `NamedOutputs` | Outputs keyed by ONNX graph output name |
| `InferenceEngine` | Lower-level runtime (power users) |
| `InferenceStats` | Per-run latency, `clankers_copies`, allocations |
| `ModelValidator` | Compare ONNX output to stored PyTorch references |
| `ValidationReport` | Numerical diff summary and deploy recommendation |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Architecture — inference stack](https://github.com/PvRao-29/clankeRS/blob/main/docs/architecture.md)
- [PyTorch → ONNX](https://github.com/PvRao-29/clankeRS/blob/main/docs/pytorch_to_onnx.md)
- [Model validation](https://github.com/PvRao-29/clankeRS/blob/main/docs/model_validation.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
