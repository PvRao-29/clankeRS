# ROS 2 Integration

clankeRS integrates with ROS 2 via the `clankers-ros2` crate.

## Default: simulation backend

By default, clankeRS uses an in-memory pub/sub bus so you can develop and test without ROS 2 installed:

```rust
use clankers::prelude::*;

let node = RobotNode::new("my_node").await?;
let sub = node.subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data()).await?;
let pub_ = node.publish::<DetectionArray>("/detections", QosProfile::default()).await?;
```

## Real ROS 2 (rclrs)

Enable the `ros2` feature on `clankers-ros2` when rclrs and ROS 2 Humble are available:

```toml
clankers-ros2 = { version = "0.1", features = ["ros2"] }
```

## QoS profiles

```rust
QosProfile::sensor_data()  // best-effort, for cameras/lidar
QosProfile::default()      // reliable, for commands/state
```
