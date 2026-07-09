# Roadmap

Published releases are on [crates.io](https://crates.io/crates/clankers) (currently v0.1.2). The items below track feature maturity inside the repo.

## v0.1 — ROS 2 node + ONNX inference
- [x] Rust workspace + crates.io publish
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

## v0.1.2 — Optimized inference (current)

Shipped on crates.io as **v0.1.2**:

- [`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) as the primary optimized inference API (`builder`, `run_named`, `run_into`, `stats`)
- Zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html) inputs via `clankers-tensor`
- Modular [`InferenceEngine`](https://docs.rs/clankers-ml/latest/clankers_ml/inference/struct.InferenceEngine.html) + ONNX Runtime backend (power-user layer)
- `clankers bench` — latency percentiles and copy/allocation accounting
- ONNX fixture integration tests + template compile checks in CI
- `camera_replay` and `perception-node` template migrated to `run_named`

**Breaking changes from v0.1.1:** `Model::run` requires `&mut self`; `InferenceStats::copies` renamed to `clankers_copies`.

## v1.0 — Production-ready SDK
- [x] Modular inference engine — backend-agnostic `InferenceEngine` over
  zero-copy `TensorView`s, with `InferenceBackend`/`BackendSession` traits, a
  `NoopBackend` and a refactored `OnnxRuntimeBackend` (zero-copy f32 input path),
  copy/allocation accounting (`InferenceStats`), preallocated-output `run_into`
  with a `Preallocate` arena for zero-alloc hot loops, `strict_realtime` build
  gating, ROS sensor adapters + composable pipeline transforms, `run_named`
  multi-input binding, [`Model`] as the primary optimized inference API over the
  engine, and a
  `clankers bench` command reporting p50/p95/p99 latency and copies/allocations.
- [ ] Stable public APIs
- [~] Full rclrs integration — backend + typed `sensor_msgs/Image` bridge
  **compiled and run against ROS 2 Humble DDS** (verified 2026-07-07;
  `ros2 topic echo` sees a real `sensor_msgs/msg/Image`), now shipped as
  **checked-in colcon packages** ([`ros2/clankers-ros2-dds`](../ros2/clankers-ros2-dds)
  + [`ros2/pubsub_minimal_dds`](../ros2/pubsub_minimal_dds)) with a one-command
  build (`scripts/setup_ros2_ws.sh`) and a graceful executor shutdown. Remaining:
  a custom `.msg` for `DetectionArray` (currently on the `std_msgs/String` JSON
  path) and broader message coverage. See [docs/ros2_integration.md](ros2_integration.md).
- [ ] Optional LibTorch / ExecuTorch backends
- [ ] Expanded geometry and runtime modules
