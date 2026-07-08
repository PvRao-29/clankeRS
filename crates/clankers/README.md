<p align="center">
  <strong>clankeRS</strong><br>
  <em>Train in PyTorch. Deploy in Rust. Replay-test against real robot logs.</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers"><img src="https://img.shields.io/crates/v/clankers.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers"><img src="https://docs.rs/clankers/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://docs.rs/clankers">API docs</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a> ·
  <a href="https://crates.io/crates/clankers-cli">CLI crate</a>
</p>

---

**clankeRS** is a Rust SDK for robotics teams on ROS 2 and PyTorch. Build memory-safe robot nodes, run ONNX inference in Rust, replay MCAP logs in tests, and ship with confidence — without replacing your existing stack.

<p align="center">
  <img
    src="https://raw.githubusercontent.com/PvRao-29/clankeRS/main/docs/assets/camera_replay.gif"
    alt="MCAP camera log replayed through ONNX inference with a latency report"
    width="640"
  >
</p>

<p align="center"><sub>Golden-path demo: MCAP → preprocess → ONNX → detections → sim pub/sub</sub></p>

## Install

```toml
# Cargo.toml
clankers = "0.1"
```

```bash
cargo add clankers
```

**Requirements:** Rust stable. First build downloads the ONNX Runtime binary automatically (network required).

For scaffolding and tooling, install the CLI separately (`clankers new` bundles templates — no clone required):

```bash
cargo install clankers-cli
```

## Quick example

A minimal perception node: subscribe to camera frames, run ONNX, publish detections.

```rust
use clankers::prelude::*;

#[clankers::node]
async fn perception(ctx: RobotContext) -> RobotResult<()> {
    let model = Model::load(ctx.resolve_path("models/detector.onnx"))?;
    let node = RobotNode::new(ctx.node_name().as_str()).await?;

    let mut images = node
        .subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;
    let detections_pub = node
        .publish::<DetectionArray>("/detections", QosProfile::default())
        .await?;

    while let Some(frame) = images.next().await {
        let tensor = ImageTensor::from_ros_msg(&frame)?
            .resize(224, 224)?
            .normalize_imagenet()?
            .to_nchw()?;

        let output = model.run(&tensor.to_vec())?;

        detections_pub
            .publish(DetectionArray {
                stamp_nanos: Timestamp::now().as_nanos(),
                frame_id: frame.frame_id.clone(),
                detections: vec![/* map model output → Detection */],
            })
            .await?;
    }
    Ok(())
}
```

Replay-test against a recorded MCAP log:

```rust
use clankers::prelude::*;

#[clankers::replay_test("tests/fixtures/camera_log.mcap")]
async fn camera_log_replays_cleanly(ctx: ReplayContext) -> RobotResult<()> {
    let result = ctx.run_replay(|_msg| async { Ok(()) }).await?;
    assert_no_panics(&result)?;
    assert_topic_exists(&result, "/camera/image_raw")?;
    Ok(())
}
```

## What you get

One dependency pulls in the full SDK surface:

| Module | Highlights |
|--------|------------|
| `clankers::ros2` | `RobotNode`, pub/sub, `ImageMsg`, `DetectionArray`, QoS profiles |
| `clankers::ml` | ONNX `Model` loading and inference |
| `clankers::tensor` | `ImageTensor` — resize, normalize, layout conversion |
| `clankers::data` | MCAP `Replay`, logging, inspection |
| `clankers::testing` | `ReplayContext`, replay assertions |
| `clankers::runtime` | `RobotRuntime`, metrics, scheduling |
| `clankers::geometry` | `Pose`, `Transform`, `Twist` |
| `clankers::prelude` | Common imports for everyday node code |

## The workflow

```text
  PyTorch model
       │
       ▼
   ONNX export ──► reference outputs (offline)
       │
       ▼
  Rust ONNX inference (clankers-ml)
       │
       ▼
  MCAP replay test (clankers-testing)
       │
       ▼
  deploy as a ROS 2 node
```

## Honest scope (v0.1)

- **Sim pub/sub works out of the box** — no ROS 2 install required for development and tests.
- **Real DDS / `rclrs`** is available from the [GitHub repo](https://github.com/PvRao-29/clankeRS) as colcon packages under `ros2/` (ROS 2 Humble). It does not ship through this crate.
- APIs are early — expect changes before v1.0.

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Getting started](https://github.com/PvRao-29/clankeRS/blob/main/docs/getting_started.md)
- [ROS 2 integration](https://github.com/PvRao-29/clankeRS/blob/main/docs/ros2_integration.md)
- [Model validation](https://github.com/PvRao-29/clankeRS/blob/main/docs/model_validation.md)
- [MCAP replay testing](https://github.com/PvRao-29/clankeRS/blob/main/docs/mcap_replay.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
