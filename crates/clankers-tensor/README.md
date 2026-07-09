<p align="center">
  <strong>clankers-tensor</strong><br>
  <em>Zero-copy tensor views and robotics preprocessing for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-tensor"><img src="https://img.shields.io/static/v1?label=crates.io&message=v0.1.4&color=orange&style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-tensor"><img src="https://docs.rs/clankers-tensor/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-tensor.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Bridge camera frames and point clouds into tensors with the preprocessing steps perception models expect — and pass them to [`Model::run_named`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html#method.run_named) as zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html)s.

## Install

```toml
clankers-tensor = "0.1"
```

```bash
cargo add clankers-tensor
```

## Example — camera frame to inference

```rust
use clankers_ros2::ImageMsg;
use clankers_tensor::{DType, ImageTensor, Layout, Shape, TensorView};

// Preprocess a ROS image message
let frame = ImageMsg::new(640, 480, vec![128u8; 640 * 480 * 3]);
let tensor = ImageTensor::from_ros_msg(&frame)?
    .resize(224, 224)?
    .normalize_imagenet()?
    .to_nchw()?;

// Zero-copy view into the preprocessed buffer (no extra clone for inference)
let shape = tensor.nchw_shape();
let view = tensor.as_nchw_view(&shape)?;
// model.run_named([("input", view)])?;

// Or bind raw sensor bytes directly (e.g. from MCAP replay)
let raw = frame.data.as_slice();
let hwc = TensorView::from_slice(
    raw,
    DType::U8,
    &Shape::from([frame.height as usize, frame.width as usize, 3]),
    Layout::HWC,
)?;
```

## Key types

| Type | Role |
|------|------|
| `TensorView` / `TensorViewMut` | Zero-copy borrowed tensor slices for inference I/O |
| `Tensor` | Owned tensor buffer |
| `ImageTensor` | `sensor_msgs`-style image → preprocessed `f32` tensor |
| `PointCloudTensor` | XYZ point cloud buffers |
| `Shape`, `Layout`, `DType` | Shape metadata, NHWC/NCHW layouts, dtypes |
| `TensorArena` | Scratch allocation for hot inference loops |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Architecture — inference stack](https://github.com/PvRao-29/clankeRS/blob/main/docs/architecture.md)
- [Camera perception tutorial](https://github.com/PvRao-29/clankeRS/blob/main/docs/camera_perception_tutorial.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
