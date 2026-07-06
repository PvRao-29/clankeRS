# Camera Perception Node Tutorial

End-to-end workflow for the clankeRS north-star demo.

## 1. Create project

```bash
clankers new camera_detector --template perception-node
cd camera_detector
```

## 2. Add model

```bash
clankers add-model models/detector.onnx
```

Or export from PyTorch:

```bash
clankers import-pytorch \
  --model ../../scripts/simple_classifier.py:SimpleClassifier \
  --checkpoint weights/model.pt \
  --output models/detector.onnx
```

## 3. Validate model

```bash
clankers validate-model --onnx models/detector.onnx --samples ../../sample_data/policy_inputs/
```

## 4. Replay sample log

```bash
clankers replay ../../sample_data/camera_log.mcap
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
clankers visualize ../../sample_data/camera_log.mcap
```

Open the MCAP file in [Foxglove Studio](https://foxglove.dev) for visualization.
