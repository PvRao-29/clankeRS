FROM ubuntu:22.04

RUN apt-get update && apt-get install -y \
    curl build-essential pkg-config libssl-dev git python3 python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# ROS 2 Humble (optional layer — comment out for lighter image)
# RUN apt-get update && apt-get install -y software-properties-common \
#     && add-apt-repository universe \
#     && curl -sSL https://raw.githubusercontent.com/ros/rosdistro/master/ros.key -o /usr/share/keyrings/ros-archive-keyring.gpg \
#     && echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/ros-archive-keyring.gpg] http://packages.ros.org/ros2/ubuntu jammy main" > /etc/apt/sources.list.d/ros2.list \
#     && apt-get update && apt-get install -y ros-humble-ros-base

WORKDIR /workspace
