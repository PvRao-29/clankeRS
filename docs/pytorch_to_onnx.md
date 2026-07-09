# PyTorch to ONNX

Export PyTorch checkpoints to ONNX for use with [`clankers::ml::Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) and `clankers validate-model`.

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

Primary inference path — zero-copy tensor views and named outputs:

```rust
use clankers::ml::OnnxRuntimeBackend;
use clankers::prelude::*;
use clankers_tensor::{DType, Layout, Shape, TensorView};

let mut model = ModelBuilder::from_config(&model_cfg, ctx.resolve_path(&model_cfg.path))?.build()?;
// or: Model::builder().backend(OnnxRuntimeBackend::default()).load("models/detector.onnx")?;

let tensor = ImageTensor::from_ros_msg(&frame)?
    .resize(224, 224)?
    .normalize_imagenet()?
    .to_nchw()?;

let shape = tensor.nchw_shape();
let view = tensor.as_nchw_view(&shape)?;
let input_name = model.engine().input_specs()[0].name.as_str();
let outputs = model.run_named([(input_name, view)])?;
```

Single-input convenience (flat `f32` buffer):

```rust
let mut model = Model::load("models/detector.onnx")?;
let output = model.run(&input_f32)?;
```

## Limitations

- Dynamic shapes may require fixed input dimensions at export time
- Custom PyTorch ops must be supported by ONNX opset
- Default opset: 17
