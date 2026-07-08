<p align="center">
  <strong>clankers-tensor</strong><br>
  <em>Robotics-focused tensor utilities for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-tensor"><img src="https://img.shields.io/crates/v/clankers-tensor.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-tensor"><img src="https://docs.rs/clankers-tensor/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-tensor.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Bridge camera frames and point clouds into `ndarray` tensors with the preprocessing steps perception models expect — resize, normalize, and layout conversion.

## Install

```toml
clankers-tensor = "0.1"
```

```bash
cargo add clankers-tensor
```

## Example

```rust
use clankers_ros2::ImageMsg;
use clankers_tensor::ImageTensor;

let frame = ImageMsg::new(640, 480, vec![128u8; 640 * 480 * 3]);

let tensor = ImageTensor::from_ros_msg(&frame)?
    .resize(224, 224)?
    .normalize_imagenet()?
    .to_nchw()?;

assert_eq!(tensor.shape(), vec![1, 3, 224, 224]);
let flat = tensor.to_vec(); // feed into ONNX
```

## Key types

| Type | Role |
|------|------|
| `ImageTensor` | `sensor_msgs`-style image → `f32` tensor |
| `PointCloudTensor` | XYZ point cloud buffers |
| `DataLayout`, `DType` | NHWC / NCHW and dtype metadata |

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
