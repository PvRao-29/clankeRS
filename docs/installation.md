# Installation

## Requirements

- Rust stable (1.75+)
- Linux recommended (Ubuntu 22.04 for ROS 2 Humble)
- Python 3.10+ (for PyTorch export scripts)

## Install clankeRS CLI

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
