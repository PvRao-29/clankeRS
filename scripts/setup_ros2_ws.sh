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

cat <<'EOF'

Done. To build clankeRS against real ROS 2:

    source ros2_ws/install/setup.bash
    cargo build -p clankers-ros2 --features ros2

Then run the smoke test (see docs/ros2_integration.md):

    cargo run -p ros2_pubsub_minimal --features ros2 &
    ros2 topic echo /camera/image_raw --no-arr
EOF
