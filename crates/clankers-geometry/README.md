<p align="center">
  <strong>clankers-geometry</strong><br>
  <em>Robotics math, transforms, and frames for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-geometry"><img src="https://img.shields.io/crates/v/clankers-geometry/0.1.3.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-geometry"><img src="https://docs.rs/clankers-geometry/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-geometry.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

`Pose`, `Transform`, and `Twist` types with serde support — the geometry layer for navigation, manipulation, and frame-aware nodes.

## Install

```toml
clankers-geometry = "0.1"
```

```bash
cargo add clankers-geometry
```

## Example

```rust
use clankers_geometry::{Pose, Transform, Twist};

let pose = Pose::identity("base_link");
let tf = Transform::new("map", "base_link");
let (x, y, z) = tf.transform_point(pose.x, pose.y, pose.z);

let mut twist = Twist::zero();
twist.linear_x = 0.5;
```

## Key types

| Type | Role |
|------|------|
| `Pose` | Position + orientation in a `frame_id` |
| `Transform` | Rigid transform between frames |
| `Twist` | Linear and angular velocity |

All types support JSON serialization for logging and replay fixtures.

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Getting started](https://github.com/PvRao-29/clankeRS/blob/main/docs/getting_started.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
