FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config libssl-dev git python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Rust (>= 1.85 for rclrs 0.7; rustup installs current stable)
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# ---------------------------------------------------------------------------
# ROS 2 Humble + ros2-rust toolchain.
#
# This lets you build the OPTIONAL `ros2` feature (real rclrs/DDS). The default
# clankeRS build (sim backend) and CI need NONE of this — plain `cargo build`
# works on the layers above. Comment this block out for a lighter image.
# ---------------------------------------------------------------------------
RUN apt-get update && apt-get install -y software-properties-common \
    && add-apt-repository universe \
    && curl -sSL https://raw.githubusercontent.com/ros/rosdistro/master/ros.key \
        -o /usr/share/keyrings/ros-archive-keyring.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/ros-archive-keyring.gpg] http://packages.ros.org/ros2/ubuntu jammy main" \
        > /etc/apt/sources.list.d/ros2.list \
    && apt-get update && apt-get install -y \
        ros-humble-ros-base \
        python3-vcstool \
        libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# colcon + cargo tooling for building ros2-rust message crates.
RUN python3 -m pip install --no-cache-dir \
        git+https://github.com/colcon/colcon-cargo.git \
        git+https://github.com/colcon/colcon-ros-cargo.git \
    && /root/.cargo/bin/cargo install --locked cargo-ament-build

# Auto-source ROS in interactive shells.
RUN echo "source /opt/ros/humble/setup.bash" >> /root/.bashrc

WORKDIR /workspace
