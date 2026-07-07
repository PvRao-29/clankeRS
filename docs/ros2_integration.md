# ROS 2 Integration

clankeRS has **two ROS 2 backends behind one API**:

| Backend | Where it lives | Needs ROS install? | Used by |
|---------|----------------|--------------------|---------|
| Simulated in-memory bus | `clankers-ros2` crate (main workspace) | No | CI, `cargo build`, all examples/tests |
| Real rclrs / DDS | `ros2/clankers-ros2-dds` (colcon package) | **Yes** (+ colcon) | Live robots, `ros2 topic` interop |

`clankers-ros2` also holds the shared, transport-agnostic message/QoS types that
both backends use. `RobotNode`, `Publisher`, `Subscriber` are identical across
both — only the transport changes.

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

> **Status: the one-command build + live DDS smoke test pass end-to-end in the
> devcontainer** (Ubuntu 22.04 + Humble, arm64; verified 2026-07-07 against the
> colcon-generated **rclrs 0.7** and **sensor_msgs 4.9.1** crates).
>
> Running `bash scripts/setup_ros2_ws.sh` builds the two checked-in colcon
> packages — [`ros2/clankers-ros2-dds`](../ros2/clankers-ros2-dds) (the rclrs/DDS
> backend) and [`ros2/pubsub_minimal_dds`](../ros2/pubsub_minimal_dds) (a minimal
> publisher) — and then:
>
> * `pubsub_minimal_dds` publishes on `/camera/image_raw`, and stock ROS tooling
>   confirms `ros2 topic type` → `sensor_msgs/msg/Image` and `ros2 topic echo`
>   shows the typed `header`/`height`/`width`/`encoding`/`step`/`data` fields
>   (**not** a JSON string) — full external DDS interop.
> * `DetectionArray` rides the `std_msgs/String` JSON path.
> * Ctrl-C exits cleanly in ~100 ms with **no** `rmw_publish.cpp` teardown
>   warning (see the `Drop` note below).
>
> **Graceful shutdown.** The executor thread spins in bounded 100 ms slices,
> polling a stop flag between them. `RobotNode`'s `Drop` sets the flag, wakes any
> in-progress slice (`ExecutorCommands::halt_spinning`), then joins the thread
> before the DDS context is torn down. The bounded slices are load-bearing: a
> plain `spin()` with no timeout never returns on live Humble DDS (even
> `halt_spinning()` does not release it), which would otherwise hang the process
> on exit.
>
> **Rust toolchain:** rclrs 0.7 needs Rust ≥ 1.85 (the devcontainer ships current
> stable). The repo's `rust-toolchain.toml` floats on `stable`, so the sim/golden
> path tracks current stable.
>
> **Rust toolchain:** rclrs 0.7 needs Rust ≥ 1.85 (the devcontainer ships current
> stable). The repo's `rust-toolchain.toml` floats on `stable`, so the sim/golden
> path tracks current stable.

### Architecture: shared core + a separate DDS package

The `clankers-ros2` crate in the main Cargo workspace is now the **ROS-free shared
core**: the in-memory sim backend plus the transport-agnostic `message`/`qos`
types. Plain `cargo build` compiles it with no ROS install.

The real DDS backend is **not** in the workspace. It is the checked-in colcon
package [`ros2/clankers-ros2-dds`](../ros2/clankers-ros2-dds), which path-depends on
`clankers-ros2` for the shared types and re-exports the same
`RobotNode`/`Publisher`/`Subscriber` API. This split is what lets the ROS-free
`cargo build --workspace` stay green while the DDS backend carries the yanked
message-crate deps — see the next section.

### Why the DDS backend is a separate colcon package (not a `cargo` feature)

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

Consequently the backend cannot be a `--features ros2` build of a main-workspace
crate — the message-crate deps it needs can't be declared anywhere the workspace
resolves without breaking the golden path. So it is a standalone package under
`ros2/` (listed in the root `Cargo.toml` `[workspace] exclude`) that is built by
colcon **inside `ros2_ws/`**, where the patch config applies. See the build steps
below.

### Build steps (Ubuntu 22.04 + ROS 2 Humble, or the repo `.devcontainer/`)

One command bootstraps the ros2-rust workspace, wires in the checked-in `ros2/`
packages, and builds the minimal publisher:

```bash
# 1. Install ROS 2 Humble (ros-humble-ros-base) — or use .devcontainer/ (recommended;
#    it installs colcon-common-extensions and builds non-interactively).
sudo apt install ros-humble-ros-base python3-vcstool libclang-dev

# 2. Bootstrap + build (from the repo root). This:
#      * clones ros2-rust and builds rclrs + std_msgs/sensor_msgs/... crates,
#      * adds the [patch.crates-io.rclrs] entry the generated config omits,
#      * symlinks ros2/clankers-ros2-dds and ros2/pubsub_minimal_dds into ros2_ws/src/,
#      * writes ros2/.cargo/config.toml (absolute patch paths — see note below),
#      * colcon-builds pubsub_minimal_dds.
source /opt/ros/humble/setup.bash
bash scripts/setup_ros2_ws.sh
```

### Smoke test (typed `sensor_msgs/Image` over DDS)

```bash
source ros2_ws/install/setup.bash
ros2 run pubsub_minimal_dds pubsub_minimal_dds &

# From a second sourced shell:
ros2 topic type /camera/image_raw     # -> sensor_msgs/msg/Image
ros2 topic echo /camera/image_raw     # -> typed header/height/width/encoding/step/data
```

A stock `ros2 topic echo` showing structured image fields (not a JSON string)
confirms external DDS interop. Ctrl-C returns cleanly (graceful `Drop`).

> **Note (patch-config discovery).** The `ros2/` packages are checked in but
> symlinked into `ros2_ws/src/` at setup time. At build time cargo canonicalizes
> the symlinked working directory back to `ros2/<pkg>`, so it would miss
> `ros2_ws/.cargo/config.toml`. The setup script therefore also writes
> `ros2/.cargo/config.toml` with the same `[patch.crates-io]` entries using
> **absolute** paths into `ros2_ws/`, discovered from the canonical package
> location. Both configs are generated and git-ignored. See
> [`ros2/README.md`](../ros2/README.md).

> **Historical note.** Before the packages were checked in, the 2026-07-07
> verification used a throwaway harness crate under `ros2_ws/src/` that
> `#[path]`-included the backend source. That harness is superseded by
> `ros2/pubsub_minimal_dds`.

### Transport note

The wire type is chosen per message by `RosMessage::wire_type()`:

| clankeRS type | ROS 2 wire type | Interop |
|---------------|-----------------|---------|
| `ImageMsg` | `sensor_msgs/Image` (via [`bridge/image.rs`](../ros2/clankers-ros2-dds/src/bridge/image.rs)) | ✅ stock ROS nodes, Foxglove, bag play |
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
