# Getting Started

clankeRS is published on [crates.io](https://crates.io/crates/clankers) at **v0.1.1**, so you can build robot nodes without cloning this repository.

## Requirements

- Rust stable (1.75+)
- Network access on first build (the ONNX Runtime binary is downloaded automatically)

Linux is recommended for ROS 2 Humble integration. For day-to-day development and tests, the in-memory sim pub/sub backend works on any platform with no ROS install.

## Install

Add the SDK to your project and install the CLI:

```bash
cargo add clankers
cargo install clankers-cli
```

Verify the CLI is on your `PATH`:

```bash
clankers --help
```

For optional ROS 2 setup, devcontainers, and development checkouts, see [Installation](installation.md).

## Create your first node

Scaffold a project from a template:

```bash
clankers new hello_clanker --template basic-node
cd hello_clanker
clankers run
```

Other templates: `perception-node`, `ml-inference-node`, `replay-test-node`.

Expected output:

```text
clankeRS node started
Subscribed to /chatter
Publishing to /chatter_out
```

New projects pull `clankers` from crates.io automatically. If you scaffold inside a clankeRS repo checkout, the template uses a local path dependency instead so you can iterate on the SDK.

### Add clankeRS to an existing project

```toml
# Cargo.toml
[dependencies]
clankers = "0.1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
```

```rust
use clankers::prelude::*;

#[clankers::node]
async fn main(ctx: RobotContext) -> Result<()> {
    let node = RobotNode::new(ctx.node_name().as_str()).await?;
    tracing::info!("node {} is running", ctx.node_name());
    Ok(())
}
```

Run with `cargo run --release` or `clankers run` from the project directory.

## Try the golden-path demo (optional)

The bundled MCAP → ONNX → detections demo and sample data live in the [GitHub repository](https://github.com/PvRao-29/clankeRS). Clone when you want to run it locally:

```bash
git clone https://github.com/PvRao-29/clankeRS.git
cd clankeRS
cargo run --release -p clankers --example camera_replay
```

This is the same pipeline as `clankers demo camera-perception`, which must be run from a repo checkout.

## What you get out of the box

- **Sim pub/sub** — develop and test nodes without installing ROS 2
- **ONNX inference** — load and run models via `clankers::ml`
- **MCAP replay** — inspect logs and write replay tests
- **CLI tooling** — scaffold projects, validate models, measure latency

Real DDS / `rclrs` integration is available from the repo under `ros2/` and builds only inside a colcon workspace. See [ROS 2 integration](ros2_integration.md).

## Next steps

- Add an ONNX model: [PyTorch to ONNX](pytorch_to_onnx.md)
- Record and replay logs: [MCAP replay](mcap_replay.md)
- Write replay tests: [Testing](testing.md)
- Validate ONNX against PyTorch references: [Model validation](model_validation.md)
