# Model Validation

`clankers validate-model` compares ONNX runtime output against a stored PyTorch reference (`expected_output.json`). PyTorch is not required at validation time — references are generated offline.

## Prerequisites

```bash
cargo install clankers-cli
```

Bundled models and reference inputs under `sample_data/` require a [repo clone](installation.md#clone-for-development-or-the-bundled-demo).

## Basic usage

```bash
clankers validate-model \
  --onnx models/policy.onnx \
  --samples path/to/inputs/ \
  --tolerance 0.001
```

With bundled sample data (from a repo clone):

```bash
clankers validate-model \
  --onnx sample_data/models/policy.onnx \
  --samples sample_data/policy_inputs/ \
  --tolerance 0.001
```

Expected output:

```text
Model compatibility: passed

Max absolute error:       0.00028
Mean absolute error:      0.00004
Rust latency p50:         3.8 ms

Status: safe to deploy
```

Place sample inputs in a directory as `input.json` — a JSON array of floats. The bundled layout is `sample_data/policy_inputs/input.json`.

## Regenerating references

From a repo clone:

```bash
pip install torch onnx
python3 scripts/make_sample_models.py
```

See also [PyTorch to ONNX](pytorch_to_onnx.md).
