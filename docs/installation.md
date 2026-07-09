# Installation

## Install from crates.io

All clankeRS crates are published on [crates.io](https://crates.io/crates/clankers) at **v0.1.2**. You do not need to clone the repository to use the SDK, scaffold projects, or run the CLI.

```bash
# Add the SDK to your project
cargo add clankers

# Install the CLI as `clankers` (templates are bundled in the binary)
cargo install clankers-cli
```

The individual crates (`clankers-core`, `clankers-data`, `clankers-ml`, `clankers-tensor`, `clankers-geometry`, `clankers-runtime`, `clankers-ros2`, `clankers-testing`, `clankers-macros`) are also on crates.io, but most projects only need the top-level `clankers` facade, which re-exports them. Use `clankers = "0.1"` in `Cargo.toml` to track the latest 0.1.x release.

> The real `rclrs`/DDS packages under `ros2/` are **not** on crates.io — they build only inside a colcon workspace. See [ROS 2 integration](ros2_integration.md).

## Requirements

- Rust stable (1.75+)
- Network access on first build (ONNX Runtime binary is downloaded automatically)
- Linux recommended (Ubuntu 22.04 for ROS 2 Humble)
- Python 3.10+ (only for PyTorch export scripts in the repo)

## Verify (crates.io install)

```bash
clankers --help
clankers new my_robot --template basic-node
```

Scaffolded projects pull `clankers` from crates.io. Inside a clankeRS repo checkout, `clankers new` uses a local path dependency instead so you can iterate on the SDK.

## Clone (for development or the bundled demo)

Clone when you need sample MCAP files, bundled ONNX models, the golden-path `camera_replay` example, ROS 2 colcon packages, or to contribute to the project:

```bash
git clone https://github.com/PvRao-29/clankeRS.git
cd clankeRS
cargo build --workspace
cargo test --workspace
```

Install the CLI from a checkout (picks up local changes immediately):

```bash
cargo install --path crates/clankers-cli
```

## Optional: ROS 2 Humble

```bash
sudo apt install ros-humble-desktop
source /opt/ros/humble/setup.bash
```

See [ROS 2 integration](ros2_integration.md) for the colcon workspace setup.

## Optional: Devcontainer

Open the repository in VS Code / Cursor and use "Reopen in Container" for a preconfigured ROS 2 + Rust environment.
