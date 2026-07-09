# Architecture

clankeRS is organized as a Cargo workspace of focused crates (package names are lowercase `clankers-*`; the product name is **clankeRS**). Workspace crates are published on [crates.io](https://crates.io/crates/clankers) at v0.1.4; most applications depend on the `clankers` facade crate. **`clankers-ffi`** exposes the inference engine over a stable C ABI; **`cpp/`** ships idiomatic C++17 wrappers in the same release.

| Crate | Responsibility |
|-------|----------------|
| `clankers` | Top-level re-exports and prelude |
| `clankers-core` | Errors, config, timestamps, RobotContext |
| `clankers-cli` | Command-line interface (`clankers bench`, `validate-model`, …) |
| `clankers-ros2` | ROS 2 pub/sub (sim backend by default) |
| `clankers-data` | MCAP read/write/replay/inspect |
| `clankers-ml` | Optimized inference — [`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) (main API), pluggable backends, validation |
| `clankers-tensor` | Zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html)s, `ImageTensor`, preprocessing pipelines |
| `clankers-testing` | Replay-based test framework |
| `clankers-macros` | clankeRS proc macros: `#[clankers::node]`, `#[clankers::replay_test]` |
| `clankers-geometry` | Poses, transforms, twists |
| `clankers-runtime` | Metrics, deadlines, queue depth |
| `clankers-ffi` | Stable C ABI over `InferenceEngine` (v0.1.4) |
| `cpp/` | C++17 RAII SDK over `clankers-ffi` (v0.1.4) |

## Inference stack

Most nodes should use **`Model`** — the user-facing optimized inference runtime:

```text
Sensor buffer (camera frame, state vector)
       │
       ▼
TensorView  (zero-copy borrow into clankeRS)
       │
       ▼
Model::run_named  (named multi-input binding, profiling)
       │
       ▼
InferenceEngine + BackendSession  (ONNX Runtime, Noop, …)
       │
       ▼
NamedOutputs  (owned tensors keyed by ONNX output name)
```

[`InferenceEngine`](https://docs.rs/clankers-ml/latest/clankers_ml/inference/struct.InferenceEngine.html) sits underneath `Model` for power users: custom backends, allocation policies (`Preallocate` arena), `run_into` with preallocated outputs, and `strict_realtime` build gating.

Copy accounting is explicit: `InferenceStats::clankers_copies` measures conversions clankeRS performs before handing tensors to the backend — not internal copies inside ONNX Runtime.

## C++ boundary (v0.1.4)

C and C++ consumers link `clankers-ffi` instead of calling Rust APIs directly:

```text
C++ application
       │
       ▼
clankers::Engine / TensorView  (cpp/)
       │
       ▼
clankers.h  (cbindgen from clankers-ffi)
       │
       ▼
InferenceEngine  (same stack as Model)
```

`run_into` exposes the robotics hot-loop path: caller-owned output buffers, per-run `InferenceStats` (latency, copies, allocations). See [cpp/README.md](../cpp/README.md).

## Data flow

```
ROS 2 topic → ImageMsg → ImageTensor → TensorView → Model → DetectionArray → ROS 2 publish
                              ↓
                         MCAP record → Replay → Tests
```

## Design principles

1. **Compatibility first** — integrate with ROS 2, PyTorch, ONNX, MCAP
2. **ONNX first** — default ML deployment path without LibTorch
3. **Optimized inference by default** — `Model` + zero-copy `TensorView`s, not a bolt-on
4. **Replay is first-class** — logs are test fixtures
5. **Boring setup** — one-command templates and clear errors
