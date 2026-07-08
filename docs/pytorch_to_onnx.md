# PyTorch to ONNX

Export PyTorch checkpoints to ONNX for use with `clankers::ml::Model` and `clankers validate-model`.

## Prerequisites

```bash
cargo install clankers-cli
cargo add clankers   # in your robot project
```

The export scripts (`scripts/`) and sample classifiers live in the [GitHub repository](installation.md#clone-for-development-or-the-bundled-demo). Clone the repo to run them locally.

## Export script

From a repo clone:

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
  --samples path/to/inputs/ \
  --tolerance 0.001
```

See [Model validation](model_validation.md) for bundled `sample_data/` paths.

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
