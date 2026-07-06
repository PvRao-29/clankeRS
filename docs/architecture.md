# Architecture

clankeRS is organized as a Cargo workspace of focused crates:

| Crate | Responsibility |
|-------|----------------|
| `clankers` | Top-level re-exports and prelude |
| `clankers-core` | Errors, config, timestamps, RobotContext |
| `clankers-cli` | Command-line interface |
| `clankers-ros2` | ROS 2 pub/sub (sim backend by default) |
| `clankers-data` | MCAP read/write/replay/inspect |
| `clankers-ml` | ONNX inference and model validation |
| `clankers-tensor` | Image and point cloud preprocessing |
| `clankers-testing` | Replay-based test framework |
| `clankers-macros` | `#[clankers::node]` and `#[clankers::replay_test]` |
| `clankers-geometry` | Poses, transforms, twists |
| `clankers-runtime` | Metrics, deadlines, queue depth |

## Data flow

```
ROS 2 topic → ImageMsg → ImageTensor → ONNX Model → DetectionArray → ROS 2 publish
                              ↓
                         MCAP record → Replay → Tests
```

## Design principles

1. **Compatibility first** — integrate with ROS 2, PyTorch, ONNX, MCAP
2. **ONNX first** — default ML deployment path without LibTorch
3. **Replay is first-class** — logs are test fixtures
4. **Boring setup** — one-command templates and clear errors
