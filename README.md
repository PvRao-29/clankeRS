# clankeRS

**Train in PyTorch. Deploy in Rust. Replay-test against real robot logs.**

clankeRS is an early-stage Rust SDK for robotics teams on ROS 2 and PyTorch. The goal is memory-safe robot nodes, ONNX inference in Rust, MCAP-based replay testing, and a CLI that ties the workflow together.

clankeRS is **not** a ROS replacement and **not** a PyTorch replacement. It is a Rust layer on top of your existing stack.

> **Honest scope today:** Pub/sub in examples uses an **in-memory simulated bus** (no ROS 2 / DDS install required). Real `rclrs`/DDS integration is written behind an off-by-default `ros2` feature but **not yet verified on a ROS box** (see the status table below). Latency numbers depend on your machine and model size — treat benchmarks as local measurements, not production guarantees.

<p align="center">
  <img src="docs/assets/camera_replay.gif" alt="clankeRS camera_replay example: MCAP log through ONNX inference with a latency report" width="680">
</p>

<p align="center"><sub>Recorded from <code>cargo run --release -p clankers --example camera_replay</code> on a sample MCAP log.</sub></p>

## Quick start

**Requirements:** Rust stable, network on first build (ONNX Runtime binary is downloaded automatically).

```bash
git clone https://github.com/PvRao-29/clankeRS.git
cd clankeRS
cargo build --workspace

# Golden-path demo (MCAP → preprocess → ONNX → detections → sim pub/sub)
cargo run --release -p clankers --example camera_replay

# Optional: install the CLI as `clankers`
cargo install --path crates/clankers-cli
clankers new hello_clanker --template basic-node
cd hello_clanker
clankers run
```

## What works today

These paths are exercised in CI (`.github/workflows/ci.yml`) from a fresh clone with `cargo` + network.

| Area | How to try it | Notes |
|------|---------------|-------|
| Workspace build | `cargo build --workspace` | |
| CLI | `cargo run -p clankers-cli -- --help` | Install with `cargo install --path crates/clankers-cli` |
| MCAP inspect | `clankers inspect sample_data/camera_log.mcap` | |
| MCAP replay (data only) | `clankers replay sample_data/camera_log.mcap` | Replays messages and reports stats; **does not** run your node or ONNX |
| MCAP latency stats | `clankers latency sample_data/camera_log.mcap` | |
| **Golden-path vertical slice** | `cargo run --release -p clankers --example camera_replay` | Full pipeline on sample data; same as `clankers demo camera-perception` |
| ONNX inference node | `cargo run -p camera_perception_node` | 10 synthetic camera frames on the sim bus; uses `sample_data/models/detector.onnx` when present |
| Model validation | see below | Compares Rust ONNX output to a **pre-recorded** PyTorch reference |
| Image preprocessing | `clankers_tensor::ImageTensor` | Resize, ImageNet normalize, NCHW |
| Replay-test macro | `#[clankers::replay_test("…")]` | See [docs/testing.md](docs/testing.md) |
| Project templates | `clankers new <name> --template basic-node` | Also: `perception-node`, `ml-inference-node`, `replay-test-node` |

### Still rough / not done yet

| Area | Status |
|------|--------|
| Real ROS 2 (DDS) via `rclrs` | Backend **compiled and run against ROS 2 Humble** (in `.devcontainer`, arm64): `ImageMsg` verified as real `sensor_msgs/msg/Image` via `ros2 topic echo`; `DetectionArray` on `std_msgs/String` JSON. Builds **only inside a colcon `ros2_ws/`** (message crates are yanked on crates.io; `rclrs` needs the git source), so it isn't a plain `--features ros2` build yet — see [docs/ros2_integration.md](docs/ros2_integration.md) |
| `clankers record` | Stub — prints a hint; MCAP recording from `clankers run` is incomplete |
| `clankers visualize` | Prints MCAP summary + Foxglove/Rerun pointers; no live bridge |
| Live PyTorch at validate time | Not implemented — validation uses committed `expected_output.json` files |
| LibTorch / ExecuTorch backends | Planned |
| Production-hardened APIs | v1.0 goal — public APIs may change |

See [docs/roadmap.md](docs/roadmap.md) for the full roadmap.

## Golden path

The workflow clankeRS is building toward:

```text
PyTorch model → ONNX export → reference outputs → Rust ONNX inference → MCAP replay test
```

**Try it now** (one command, sample data included):

```bash
cargo run --release -p clankers --example camera_replay
```

This reads `sample_data/camera_log.mcap` and `sample_data/models/detector.onnx`, runs preprocess → ONNX → detections, publishes on a **simulated** `/detections` topic, and prints measured latency. Example output (numbers vary by machine):

```text
Loading detector.onnx...
Opening camera_log.mcap...
Running replay...

Frame 200/200

Published 200 detections to /detections
Replay complete.

Replay Summary
  Frames:    200
  FPS:       …
  Detections received on /detections: 200
  Dropped:   0

Latency:
  p50: …
  p95: …
  p99: …

✓ Replay passed
```

The FPS/latency figures measure the in-process pipeline (decode → preprocess → ONNX → sim publish). They do **not** include real camera I/O, network DDS, or robot hardware.

## Model validation

`validate-model` checks that Rust ONNX Runtime output matches a **stored PyTorch reference** (`expected_output.json`) for a fixed sample input. PyTorch is **not** required at validation time — references are generated offline (see below).

```bash
cargo run -p clankers-cli -- validate-model \
  --onnx sample_data/models/detector.onnx \
  --samples sample_data/detector_inputs
```

Example output:

```text
Model compatibility: passed

PyTorch output shape:     [1, 6]
Rust ONNX output shape:   [1, 6]

Max absolute error:       …
Mean absolute error:      …
Tolerance:                0.001000

Rust latency p50:         … ms

Status: safe to deploy
```

The `safe to deploy` line means **numerical agreement on the bundled sample inputs** within tolerance — not a full production sign-off. For your own models, regenerate references and tighten tolerance as needed. Live `--pytorch` / `--checkpoint` comparison flags are reserved for a future release.

Policy model (default samples dir):

```bash
cargo run -p clankers-cli -- validate-model --onnx sample_data/models/policy.onnx
```

## Regenerating sample models

Sample ONNX models and PyTorch reference outputs live under `sample_data/` and are checked into the repo:

```bash
pip install torch onnx
python3 scripts/make_sample_models.py
```

This exports two small deterministic PyTorch models to ONNX and writes `expected_output.json` for `validate-model`.

## Documentation

- [Getting started](docs/getting_started.md)
- [Installation](docs/installation.md)
- [ROS 2 integration](docs/ros2_integration.md) — sim bus today, real DDS planned
- [PyTorch to ONNX](docs/pytorch_to_onnx.md)
- [Model validation](docs/model_validation.md)
- [MCAP replay](docs/mcap_replay.md)
- [Testing](docs/testing.md)
- [Architecture](docs/architecture.md)

## License

MIT — see [LICENSE](LICENSE).
