# Installation

## Install from crates.io

All Rust SDK crates are published on [crates.io](https://crates.io/crates/clankers) at **v0.1.4**, including [`clankers-ffi`](https://crates.io/crates/clankers-ffi). You do not need to clone the repository to use the SDK, scaffold projects, run the CLI, or build the C ABI.

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

## Optional: C++ SDK (v0.1.4)

To build ONNX inference from C++ using the same engine as Rust:

- [`clankers-ffi`](https://crates.io/crates/clankers-ffi) in your Cargo workspace (builds `libclankers_ffi`)
- CMake 3.16+
- C++17 compiler (`g++` or `clang++`)

```bash
cargo add clankers-ffi
bash scripts/build_cpp_sdk.sh
./cpp/build/minimal_inference path/to/model.onnx
```

Details: [cpp/README.md](../cpp/README.md). For deploy-time robots without a Rust toolchain, ship prebuilt `libclankers_ffi` artifacts and headers from your release build.

## Optional: Devcontainer

Open the repository in VS Code / Cursor and use "Reopen in Container" for a preconfigured ROS 2 + Rust environment.
