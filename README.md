# clankeRS

clankeRS is a Rust SDK for building reliable robotics applications inside the existing ROS 2 ecosystem.

It helps you write Rust robot nodes that can subscribe to ROS 2 topics, run PyTorch-trained models through ONNX, publish outputs, record MCAP logs, and replay real robot data in tests.

clankeRS is not a ROS replacement and not a PyTorch replacement. It is a production-focused Rust layer for teams that want memory safety, predictable performance, and better testing in robotics software.

## Who is it for?

Robotics developers and ML engineers who work with ROS 2, Python, PyTorch, and MCAP logs — and want to adopt Rust for runtime nodes without abandoning their existing stack.

## Quick start

```bash
# Build everything
cargo build --workspace

# Install the CLI (required for `clankers` commands)
cargo install --path crates/clankers-cli

# Or run CLI without installing:
# cargo run -p clankers-cli -- <command>

# Create a new node
clankers new hello_robot --template basic-node
cd hello_robot
clankers run
```

### Run examples from the repo

```bash
cargo run -p camera_perception_node
cargo run -p ros2_pubsub_minimal
cargo run -p clankers-cli -- inspect sample_data/camera_log.mcap
cargo run -p clankers-cli -- replay sample_data/camera_log.mcap
```

## North-star demo

```bash
clankers new camera_detector --template perception-node
clankers add-model models/detector.onnx
clankers replay sample_data/camera_log.mcap
clankers test
clankers run
```

## What works today

- Rust workspace with modular crates
- `clankeRS.toml` configuration loading
- Simulated ROS 2 pub/sub (no ROS install required for dev)
- MCAP inspect, replay, compare, and latency reporting
- ONNX model loading and inference (via ONNX Runtime)
- Image tensor preprocessing (resize, ImageNet normalize, NCHW)
- Replay-based testing helpers and proc macros
- CLI: `new`, `run`, `test`, `inspect`, `replay`, `validate-model`, `import-pytorch`

## What is planned

- Full rclrs integration behind `ros2` feature flag
- LibTorch / ExecuTorch optional backends
- Foxglove and Rerun visualization bridges
- Geometry and runtime observability expansion

See [docs/roadmap.md](docs/roadmap.md) for details.

## Documentation

- [Getting started](docs/getting_started.md)
- [Installation](docs/installation.md)
- [ROS 2 integration](docs/ros2_integration.md)
- [PyTorch to ONNX](docs/pytorch_to_onnx.md)
- [MCAP replay](docs/mcap_replay.md)
- [Testing](docs/testing.md)
- [Architecture](docs/architecture.md)

## License

MIT — see [LICENSE](LICENSE).
