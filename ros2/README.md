# Real ROS 2 (rclrs/DDS) packages

These packages are the **real DDS backend** for clankeRS. They are **not** part of
the main Cargo workspace (`cargo build --workspace` never touches them) and can
only be built by **colcon inside a ROS 2 workspace**, because the ROS message
crates they need (`sensor_msgs`, `std_msgs`, `builtin_interfaces`) are yanked on
crates.io and `rclrs` must come from the ros2-rust git source. See
[../docs/ros2_integration.md](../docs/ros2_integration.md) for the full rationale.

| Package | What it is |
|---------|-----------|
| [`clankers-ros2-dds/`](clankers-ros2-dds) | The rclrs/DDS backend. Path-depends on the workspace crate `clankers-ros2` for the shared, transport-agnostic `message`/`qos` types and re-exports `RobotNode`/`Publisher`/`Subscriber` with the **same API** as the sim backend. |
| [`pubsub_minimal_dds/`](pubsub_minimal_dds) | A minimal publisher that puts a real `sensor_msgs/msg/Image` on `/camera/image_raw` — the smoke-test target. |

## One-command build (Ubuntu 22.04 + ROS 2 Humble, or the repo `.devcontainer/`)

```bash
source /opt/ros/humble/setup.bash
bash scripts/setup_ros2_ws.sh          # bootstrap ros2-rust + build pubsub_minimal_dds
```

`setup_ros2_ws.sh` (from the repo root) bootstraps the ros2-rust workspace, builds
the message crates, **symlinks these packages into `ros2_ws/src/`**, generates the
patch config they need, and colcon-builds `pubsub_minimal_dds`.

## Smoke test (typed image over DDS)

```bash
source ros2_ws/install/setup.bash
ros2 run pubsub_minimal_dds pubsub_minimal_dds &

# From a second shell that has also sourced ros2_ws/install/setup.bash:
ros2 topic type /camera/image_raw      # -> sensor_msgs/msg/Image
ros2 topic echo /camera/image_raw      # -> typed header/height/width/encoding/step/data
```

Seeing a real `sensor_msgs/msg/Image` (structured fields, **not** a JSON string)
from a stock `ros2 topic echo` confirms end-to-end DDS interop. Ctrl-C the
publisher; it returns cleanly (the node halts and joins the rclrs executor thread
before context teardown, so there is no `rmw_publish.cpp` exit warning).

## How the build finds the message crates (implementation note)

The packages are checked in under `ros2/` but **symlinked** into `ros2_ws/src/` at
setup time. At build time cargo canonicalizes the symlinked working directory back
to `ros2/<pkg>`, so it would walk up from there and miss `ros2_ws/.cargo/config.toml`
(the colcon-generated `[patch.crates-io]` for the message crates). The setup script
therefore also writes `ros2/.cargo/config.toml` — the same patches with **absolute**
paths into `ros2_ws/` — which is discovered from the canonical package location.
Both files are generated and git-ignored (`ros2_ws/`, `ros2/.cargo/`).

The relative path deps (`clankers-ros2-dds` -> `../../crates/clankers-ros2`) resolve
against the canonical manifest location, i.e. the real repo, so they work through
the symlink.
