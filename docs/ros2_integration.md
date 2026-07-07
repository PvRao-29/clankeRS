# ROS 2 Integration

clankeRS talks to ROS 2 through the `clankers-ros2` crate, which has **two
backends behind one API** selected at compile time:

| Backend | Feature | Needs ROS install? | Used by |
|---------|---------|--------------------|---------|
| Simulated in-memory bus | `sim` (default) | No | CI, `cargo build`, all examples/tests |
| Real rclrs / DDS | `ros2` | **Yes** (+ colcon) | Live robots, `ros2 topic` interop |

`RobotNode`, `Publisher`, `Subscriber` are identical across both — only the
transport changes.

```rust
use clankers::prelude::*;

let node = RobotNode::new("my_node").await?;
let mut sub = node.subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data()).await?;
let det = node.publish::<DetectionArray>("/detections", QosProfile::default()).await?;
```

## Default: simulation backend

Nothing to install. This is what `cargo build`, CI, and the
[`camera_replay`](../crates/clankers/examples/camera_replay.rs) golden path use.
Pub/sub is an in-memory broadcast bus; messages are serialized as JSON.

## Real ROS 2 (rclrs) — status

> **Status: compiled and run against real ROS 2 Humble (DDS).** On 2026-07-07 the
> backend ([`rclrs_backend.rs`](../crates/clankers-ros2/src/rclrs_backend.rs)) and
> the `sensor_msgs/Image` bridge ([`bridge/image.rs`](../crates/clankers-ros2/src/bridge/image.rs))
> were built inside the [`.devcontainer`](../.devcontainer) image (Ubuntu 22.04 +
> Humble, arm64) against the colcon-generated **rclrs 0.7** and **sensor_msgs 4.9.1**
> crates, and exercised over live DDS:
>
> * `ImageMsg` publishes/subscribes as a real `sensor_msgs/msg/Image` — confirmed
>   with `ros2 topic type` / `ros2 topic echo` from a stock ROS node (typed
>   `height`/`width`/`encoding`/`step`/`data`, **not** a JSON string).
> * `DetectionArray` round-trips over the `std_msgs/String` JSON path.
> * The "subscriptions created after `executor.spin()` starts" ordering (see
>   `RobotNode::new`) **does** deliver — messages flow. (One benign
>   `cannot publish data … rmw_publish.cpp` line can print at process exit because
>   the detached executor thread is still spinning during context teardown; a
>   graceful shutdown/`Drop` is a follow-up polish item.)
>
> **Rust toolchain:** rclrs 0.7 needs Rust ≥ 1.85 (the devcontainer ships current
> stable, 1.96 at time of writing). The repo's `rust-toolchain.toml` floats on
> `stable`, so the sim/golden path tracks current stable.

### Why the `ros2` feature only builds inside the colcon workspace

Two crates.io realities force this — both found the hard way:

1. **The message crates are not usable from crates.io.** `sensor_msgs`,
   `std_msgs`, and `builtin_interfaces` are generated per-distro by `colcon` +
   `rosidl_generator_rs`; the crates.io names are **yanked** (e.g. `sensor_msgs
   4.2.3`). Listing them in `Cargo.toml` breaks the ROS-free `cargo build` with
   "failed to select a version … is yanked". colcon's generated
   `install/.cargo/config.toml` supplies them via `[patch.crates-io]` path
   entries — so they resolve **only** inside `ros2_ws/`.
2. **crates.io `rclrs` is API-incompatible with the ros2-rust `main` rclrs.**
   Both call themselves `0.7.0`, but the published one is older (e.g. it has no
   `Context::create_basic_executor`). colcon builds the **git** rclrs; a downstream
   package must therefore add a `[patch.crates-io] rclrs = { path = ".../ros2_rust/rclrs" }`
   (the generated config does **not** patch `rclrs`).

Consequently the backend cannot be compiled from the main Cargo workspace with a
plain `cargo build -p clankers-ros2 --features ros2` — the message-crate deps it
needs can't be declared there without breaking the golden path. It must be built
as a package **inside `ros2_ws/`**, where the patch config applies. See
"Reference build" below.

### Build steps (Ubuntu 22.04 + ROS 2 Humble)

```bash
# 1. Install ROS 2 Humble (ros-humble-ros-base) — or use .devcontainer/ (recommended;
#    it now installs colcon-common-extensions and builds non-interactively).
sudo apt install ros-humble-ros-base python3-vcstool libclang-dev

# 2. Bootstrap the ros2-rust workspace (generates rclrs + std_msgs/sensor_msgs/... crates
#    and ros2_ws/.cargo/config.toml with the [patch.crates-io] entries).
source /opt/ros/humble/setup.bash
bash scripts/setup_ros2_ws.sh

# 3. Add the rclrs patch the generated config omits, then build a package that
#    lives *inside* ros2_ws/ (so it picks up ros2_ws/.cargo/config.toml).
cat >> ros2_ws/.cargo/config.toml <<'EOF'

[patch.crates-io.rclrs]
path = "src/ros2_rust/rclrs"
EOF
```

> **Note (packaging is still open):** wiring `clankers-ros2` itself into `ros2_ws/`
> as an `ament_cargo` package (or splitting the DDS backend into its own crate that
> lives there) is the remaining task — the current `clankers-ros2` is a member of
> the main Cargo workspace and can't carry the message-crate deps. The verification
> that the backend *code* is correct was done with a small harness crate placed in
> `ros2_ws/src/` that `#[path]`-includes the real backend source; see below.

### Reference build (what was actually verified)

Because `clankers-ros2` isn't wired into `ros2_ws/` yet, the backend was verified
with a small harness crate under `ros2_ws/src/` that pulls the **real** backend
source in via `#[path]` and builds it against the colcon crates:

```toml
# ros2_ws/src/<harness>/Cargo.toml
[dependencies]
rclrs = "0.7"                       # redirected to the git rclrs by the patch above
std_msgs = "*"
sensor_msgs = "*"
builtin_interfaces = "*"
# + async-trait, serde, serde_json, tokio, tracing, thiserror
# + a tiny clankers-core shim providing RobotError / RobotResult / Timestamp
```

```rust
// src/lib.rs — compile the real files verbatim
#[path = ".../clankers-ros2/src/qos.rs"]           pub mod qos;
#[path = ".../clankers-ros2/src/message.rs"]       pub mod message;
pub mod bridge { #[path = ".../bridge/image.rs"]   pub mod image; }
#[path = ".../clankers-ros2/src/rclrs_backend.rs"] pub mod rclrs_backend;
```

```bash
source /opt/ros/humble/setup.bash
cd ros2_ws/src/<harness> && cargo build      # compiles against rclrs 0.7 + sensor_msgs 4.9.1
./target/debug/<pub-bin> &                   # publishes ImageMsg on a topic

# Confirm the wire type from a stock ROS node:
ros2 topic type /verify/image                # -> sensor_msgs/msg/Image
ros2 topic echo /verify/image                # -> header/height/width/encoding/step/data
```

This is exactly how the 2026-07-07 verification was run (see the status note
above). Once `clankers-ros2` is packaged into `ros2_ws/`, the intended end-user
smoke test is `cargo run -p ros2_pubsub_minimal` + `ros2 topic echo
/camera/image_raw`.

### Transport note

The wire type is chosen per message by `RosMessage::wire_type()`:

| clankeRS type | ROS 2 wire type | Interop |
|---------------|-----------------|---------|
| `ImageMsg` | `sensor_msgs/Image` (via [`bridge/image.rs`](../crates/clankers-ros2/src/bridge/image.rs)) | ✅ stock ROS nodes, Foxglove, bag play |
| `DetectionArray` (and any other type) | `std_msgs/String` holding JSON | ⚠️ clankeRS ↔ clankeRS only |

`ImageMsg` publishes/subscribes as a real typed `sensor_msgs/Image`, so
`ros2 topic echo` shows structured image fields and stock nodes interoperate.
Everything else defaults to a `std_msgs/String` JSON envelope, which keeps the
backend generic. Giving `DetectionArray` first-class interop needs a custom
`.msg` (e.g. `clankers_msgs/DetectionArray` or `vision_msgs/Detection2DArray`) —
tracked in the roadmap.

## QoS profiles

```rust
QosProfile::sensor_data()  // best-effort, shallow depth — cameras/lidar
QosProfile::default()      // reliable, volatile — commands/state
```

Under the `ros2` backend these map to rclrs reliability/durability/depth; under
`sim` they are currently advisory.
