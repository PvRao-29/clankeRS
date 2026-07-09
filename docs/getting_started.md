# Getting Started

clankeRS is published on [crates.io](https://crates.io/crates/clankers) at **v0.1.4**, so you can build robot nodes without cloning this repository.

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
- **Optimized ONNX inference** — load models with [`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html), bind zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html) inputs, read named outputs
- **MCAP replay** — inspect logs and write replay tests
- **CLI tooling** — scaffold projects, validate models, benchmark latency (`clankers bench`)
- **C++ inference SDK** — [`clankers-ffi`](https://crates.io/crates/clankers-ffi) C ABI + `cpp/` C++17 wrappers over the same ONNX engine; see [cpp/README.md](../cpp/README.md)

Real DDS / `rclrs` integration is available from the repo under `ros2/` and builds only inside a colcon workspace. See [ROS 2 integration](ros2_integration.md).

### Run inference (primary API)

```rust
use clankers::ml::OnnxRuntimeBackend;
use clankers::prelude::*;
use clankers_tensor::{DType, Layout, Shape, TensorView};

let mut model = Model::builder()
    .backend(OnnxRuntimeBackend::default())
    .load("models/policy.onnx")?;

let image = TensorView::from_slice(
    image_bytes,
    DType::U8,
    &Shape::from([1, 64, 64, 3]),
    Layout::Contiguous,
)?;
let state = TensorView::from_f32(&state_f32, &Shape::from([1, 12]))?;

let outputs = model.run_named([("image", image), ("state", state)])?;
```

Single-input convenience (templates and quick scripts):

```rust
let mut model = Model::load("models/policy.onnx")?;
let action = model.run(&input_f32)?;
```

## Next steps

- Add an ONNX model: [PyTorch to ONNX](pytorch_to_onnx.md)
- Record and replay logs: [MCAP replay](mcap_replay.md)
- Write replay tests: [Testing](testing.md)
- Validate ONNX against PyTorch references: [Model validation](model_validation.md)
- Use inference from C++: [C++ SDK](../cpp/README.md)
