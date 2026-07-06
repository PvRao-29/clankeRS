# PyTorch to ONNX

## Export script

```bash
python3 scripts/export_pytorch_to_onnx.py \
  --model scripts/simple_classifier.py:SimpleClassifier \
  --checkpoint weights/model.pt \
  --output models/detector.onnx
```

Or via CLI:

```bash
clankers import-pytorch \
  --model scripts/simple_classifier.py:SimpleClassifier \
  --checkpoint weights/model.pt \
  --output models/detector.onnx
```

## Validate before deployment

```bash
clankers validate-model \
  --onnx models/detector.onnx \
  --samples sample_data/policy_inputs/ \
  --tolerance 0.001
```

## Use in a node

```toml
[model.detector]
source_framework = "pytorch"
path = "models/detector.onnx"
backend = "onnxruntime"
device = "cpu"
```

```rust
let model = Model::load("models/detector.onnx")?;
let output = model.run(&tensor.to_vec())?;
```

## Limitations

- Dynamic shapes may require fixed input dimensions at export time
- Custom PyTorch ops must be supported by ONNX opset
- Default opset: 17
