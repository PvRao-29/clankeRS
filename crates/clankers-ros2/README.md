<p align="center">
  <strong>clankers-ros2</strong><br>
  <em>ROS-free sim backend and shared message types for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-ros2"><img src="https://img.shields.io/static/v1?label=crates.io&message=v0.1.4&color=orange&style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-ros2"><img src="https://docs.rs/clankers-ros2/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-ros2.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/docs/ros2_integration.md">ROS 2 integration</a>
</p>

---

Pub/sub for robot nodes with **no ROS 2 install required** — an in-memory sim bus for development, CI, and replay tests. Also defines the shared `ImageMsg`, `DetectionArray`, and `QosProfile` types used by the real DDS backend in the GitHub repo.

## Install

```toml
clankers-ros2 = "0.1"
```

```bash
cargo add clankers-ros2
```

## Example

```rust
use clankers_ros2::{ImageMsg, QosProfile, RobotNode};

#[tokio::main]
async fn main() -> clankers_core::RobotResult<()> {
    let node = RobotNode::new("camera").await?;

    let mut sub = node
        .subscribe::<ImageMsg>("/camera/image_raw", QosProfile::sensor_data())
        .await?;
    let pub_det = node
        .publish::<ImageMsg>("/camera/processed", QosProfile::default())
        .await?;

    while let Some(frame) = sub.next().await {
        pub_det.publish(frame).await?;
    }
    Ok(())
}
```

## Key types

| Type | Role |
|------|------|
| `RobotNode` | Create publishers and subscribers |
| `ImageMsg`, `DetectionArray` | Camera and perception messages |
| `QosProfile` | Reliability, durability, and sensor presets |
| `inject_message` | Feed the sim bus during tests |

## Real DDS backend

The `rclrs`/DDS implementation is **not** in this crate (ROS message deps are yanked on crates.io). It ships as colcon packages under [`ros2/`](https://github.com/PvRao-29/clankeRS/tree/main/ros2) in the main repository.

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [ROS 2 integration](https://github.com/PvRao-29/clankeRS/blob/main/docs/ros2_integration.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
