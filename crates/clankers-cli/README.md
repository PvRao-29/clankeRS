<p align="center">
  <strong>clankers-cli</strong><br>
  <em>Command-line interface for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-cli"><img src="https://img.shields.io/crates/v/clankers-cli.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-cli"><img src="https://docs.rs/clankers-cli/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-cli.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Scaffold projects, replay MCAP logs, validate ONNX models, and run the camera-perception demo — the `clankers` binary for day-to-day robotics workflows.

## Install

```bash
cargo install clankers-cli
```

```bash
clankers --help
```

## Quick start

```bash
# New project from a template
clankers new my_robot --template perception-node
cd my_robot
clankers run

# Inspect and replay a log
clankers inspect sample_data/camera_log.mcap
clankers replay sample_data/camera_log.mcap

# Validate ONNX against stored PyTorch references
clankers validate-model \
  --onnx sample_data/models/detector.onnx \
  --samples sample_data/detector_inputs

# Golden-path demo (MCAP → ONNX → detections)
clankers demo camera-perception
```

## Commands

| Command | Purpose |
|---------|---------|
| `new` | Scaffold from `basic-node`, `perception-node`, `ml-inference-node`, or `replay-test-node` |
| `run` | Run the current project's node |
| `test` | Run replay tests and `cargo test` |
| `inspect` | Summarize an MCAP file |
| `replay` | Replay a log (optionally through a node) |
| `latency` | Latency stats from a replay |
| `compare` | Diff two MCAP files |
| `validate-model` | ONNX vs PyTorch reference outputs |
| `import-pytorch` | Export a checkpoint to ONNX |
| `add-model` | Register a model in `clankeRS.toml` |
| `visualize` | MCAP summary for Foxglove / Rerun |
| `demo` | Run bundled demos (`camera-perception`) |
| `record` | MCAP recording (stub in v0.1) |

## Learn more

- [Getting started](https://github.com/PvRao-29/clankeRS/blob/main/docs/getting_started.md)
- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
