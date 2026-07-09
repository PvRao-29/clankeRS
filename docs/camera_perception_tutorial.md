# Camera Perception Node Tutorial

End-to-end workflow for the clankeRS north-star demo.

## Prerequisites

Install from [crates.io](https://crates.io/crates/clankers):

```bash
cargo install clankers-cli
```

For steps that use bundled sample logs and models (`sample_data/`, export scripts), clone the repository — that data is not shipped with the crates.io packages. See [Installation](installation.md#clone-for-development-or-the-bundled-demo).

## 1. Create project

From any directory (no clone required):

```bash
clankers new camera_detector --template perception-node
cd camera_detector
```

## 2. Add model

Copy or export an ONNX model into your project:

```bash
clankers add-model models/detector.onnx
```

Export from PyTorch (requires a repo clone for the sample script):

```bash
# from the clankeRS repo root
clankers import-pytorch \
  --model scripts/simple_classifier.py:SimpleClassifier \
  --checkpoint weights/model.pt \
  --output models/detector.onnx
```

## 3. Validate model

Against your own sample inputs:

```bash
clankers validate-model --onnx models/detector.onnx --samples path/to/inputs/
```

With the bundled detector and reference inputs (from a repo clone):

```bash
clankers validate-model \
  --onnx sample_data/models/detector.onnx \
  --samples sample_data/detector_inputs/
```

## 4. Replay sample log

Your own MCAP file:

```bash
clankers replay path/to/camera_log.mcap
```

Bundled sample log (from a repo clone):

```bash
clankers replay sample_data/camera_log.mcap
```

## 5. Run tests

```bash
clankers test
```

## 6. Run live node

```bash
clankers run
```

## Visualization

```bash
clankers visualize sample_data/camera_log.mcap
```

Open the MCAP file in [Foxglove Studio](https://foxglove.dev) for visualization.

## Golden-path demo

For the full MCAP → preprocess → ONNX → detections pipeline on bundled data, clone the repo and run:

```bash
cargo run --release -p clankers --example camera_replay
# or, from the repo root:
clankers demo camera-perception
```

The `camera_replay` example binds camera frames as zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html)s via `Model::run_named` — the same pattern used in the `perception-node` template.

See [Getting started](getting_started.md) for the crates.io-first setup.
