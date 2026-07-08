# Roadmap

Published releases are on [crates.io](https://crates.io/crates/clankers) (currently v0.1.1). The items below track feature maturity inside the repo.

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

## v1.0 — Production-ready SDK
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
