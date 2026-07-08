# Installation

## Install from crates.io

All clankeRS crates are published on [crates.io](https://crates.io/crates/clankers) under the `0.1.0` release, so you don't need to clone the repo to use the SDK or the CLI.

```bash
# Add the SDK to your project
cargo add clankers

# Install the CLI as `clankers`
cargo install clankers-cli
```

The individual crates (`clankers-core`, `clankers-data`, `clankers-ml`, `clankers-tensor`, `clankers-geometry`, `clankers-runtime`, `clankers-ros2`, `clankers-testing`, `clankers-macros`) are also available, but most projects only need the top-level `clankers` facade, which re-exports them.

> The real `rclrs`/DDS packages under `ros2/` are not on crates.io — they build only inside a colcon workspace. See [ROS 2 integration](ros2_integration.md).

## Clone (for development or the bundled demo)

```bash
git clone https://github.com/PvRao-29/clankeRS.git
cd clankeRS
```

## Requirements

- Rust stable (1.75+)
- Linux recommended (Ubuntu 22.04 for ROS 2 Humble)
- Python 3.10+ (for PyTorch export scripts)

## Install clankeRS CLI (from a checkout)

```bash
cargo install --path crates/clankers-cli
```

## Optional: ROS 2 Humble

```bash
sudo apt install ros-humble-desktop
source /opt/ros/humble/setup.bash
```

## Optional: Devcontainer

Open the repository in VS Code / Cursor and use "Reopen in Container" for a preconfigured ROS 2 + Rust environment.

## Verify

```bash
clankers --help
cargo test --workspace
```
