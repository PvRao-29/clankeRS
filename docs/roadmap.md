# Roadmap

## v0.1 — ROS 2 node + ONNX inference
- [x] Rust workspace
- [x] CLI skeleton
- [x] Basic node template
- [x] ONNX model loading
- [x] Config loading

## v0.2 — MCAP logging and replay
- [x] MCAP read/write
- [x] `clankers inspect` / `replay`
- [x] Sample data

## v0.3 — Model validation
- [x] PyTorch export script
- [x] `clankers validate-model`
- [x] Numerical tolerance reporting

## v0.4 — Replay-based testing
- [x] `clankers-testing` crate
- [x] `#[clankers::replay_test]` macro
- [x] Assertion helpers

## v0.5 — Polished perception demo
- [x] `camera_perception_node` example
- [x] Perception node template
- [x] End-to-end tutorial

## v1.0 — Production-ready SDK
- [ ] Stable public APIs
- [ ] Full rclrs integration
- [ ] Optional LibTorch / ExecuTorch backends
- [ ] Expanded geometry and runtime modules
