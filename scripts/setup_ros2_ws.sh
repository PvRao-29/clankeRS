#!/usr/bin/env bash
#
# Bootstrap a ROS 2 (Humble) + ros2-rust workspace so clankeRS can be built with
# the `ros2` feature. This is ONLY needed for real DDS — the default `sim`
# backend and all of CI build with plain `cargo` and no ROS install.
#
# Prereqs: Ubuntu 22.04 with ROS 2 Humble installed (`ros-humble-ros-base`),
# `python3-vcstool`, `libclang-dev`, and a Rust toolchain >= 1.85.
#
# Usage:
#   source /opt/ros/humble/setup.bash
#   bash scripts/setup_ros2_ws.sh
#   source ros2_ws/install/setup.bash
#   cargo build -p clankers-ros2 --features ros2   # (from the repo root)
#
# See docs/ros2_integration.md for the full, verified-on-a-ROS-box flow.

set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WS="$REPO/ros2_ws"

if [ -z "${ROS_DISTRO:-}" ]; then
  echo "error: source your ROS 2 install first (e.g. 'source /opt/ros/humble/setup.bash')" >&2
  exit 1
fi

echo "==> Installing colcon + cargo-ament-build tooling"
python3 -m pip install --user \
  git+https://github.com/colcon/colcon-cargo.git \
  git+https://github.com/colcon/colcon-ros-cargo.git
cargo install --locked cargo-ament-build

echo "==> Creating ros2-rust workspace at $WS"
mkdir -p "$WS/src"
if [ ! -d "$WS/src/ros2_rust" ]; then
  git clone https://github.com/ros2-rust/ros2_rust.git "$WS/src/ros2_rust"
fi

echo "==> Importing message-generation dependencies"
vcs import "$WS/src" < "$WS/src/ros2_rust/ros2_rust_humble.repos"

echo "==> Building the ROS message crates (rclrs, std_msgs, sensor_msgs, ...)"
cd "$WS"
colcon build --packages-up-to sensor_msgs std_msgs rclrs

# colcon's generated .cargo/config.toml patches the message crates but NOT rclrs.
# The crates.io rclrs 0.7.0 is an older, API-incompatible release than the
# ros2-rust `main` rclrs colcon just built, so downstream cargo packages must
# patch rclrs to the local git source. Add it if missing.
CARGO_CFG="$WS/.cargo/config.toml"
if [ -f "$CARGO_CFG" ] && ! grep -q "patch.crates-io.rclrs" "$CARGO_CFG"; then
  echo "==> Adding [patch.crates-io.rclrs] -> src/ros2_rust/rclrs to $CARGO_CFG"
  cat >> "$CARGO_CFG" <<'EOF'

[patch.crates-io.rclrs]
path = "src/ros2_rust/rclrs"
EOF
fi

cat <<'EOF'

Done — rclrs + std_msgs/sensor_msgs Rust crates are built and patched in
ros2_ws/.cargo/config.toml.

IMPORTANT: the real DDS backend must be built as a cargo package that lives
*inside* ros2_ws/ (so it inherits ros2_ws/.cargo/config.toml). It cannot be built
from the main clankeRS workspace with `cargo build --features ros2`, because the
message crates are yanked on crates.io and can't be declared there without
breaking the ROS-free build. Wiring clankers-ros2 into ros2_ws/ is tracked in
docs/ros2_integration.md.

To reproduce the verified end-to-end check (typed sensor_msgs/Image over DDS),
see the "Reference build" section of docs/ros2_integration.md.
EOF
