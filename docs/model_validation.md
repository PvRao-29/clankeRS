# Model Validation

`clankers validate-model` compares ONNX runtime output against a PyTorch reference.

```bash
clankers validate-model \
  --onnx models/policy.onnx \
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

Place sample inputs in `sample_data/policy_inputs/input.json` as a JSON array of floats.
