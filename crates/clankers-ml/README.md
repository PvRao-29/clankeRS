<p align="center">
  <strong>clankers-ml</strong><br>
  <em>ML inference and model deployment for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-ml"><img src="https://img.shields.io/crates/v/clankers-ml.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-ml"><img src="https://docs.rs/clankers-ml/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-ml.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Load ONNX models, run inference with latency tracking, and validate Rust output against offline PyTorch references — the deploy side of the PyTorch → Rust pipeline.

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
use clankers_ml::{Model, ModelValidator};

let model = Model::load("models/detector.onnx")?;
let output = model.run(&input_tensor)?;

let report = ModelValidator::new()
    .onnx_model("models/detector.onnx")
    .sample_inputs("fixtures/detector_inputs")
    .tolerance(0.001)
    .validate()?;
report.print();
```

## Key types

| Type | Role |
|------|------|
| `Model` | Load and run an ONNX model |
| `ModelBuilder` | Configure backend, warmup, latency limits |
| `ModelValidator` | Compare ONNX output to stored PyTorch references |
| `ValidationReport` | Numerical diff summary and deploy recommendation |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [PyTorch → ONNX](https://github.com/PvRao-29/clankeRS/blob/main/docs/pytorch_to_onnx.md)
- [Model validation](https://github.com/PvRao-29/clankeRS/blob/main/docs/model_validation.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
