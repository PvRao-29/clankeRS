# Roadmap

Published releases are on [crates.io](https://crates.io/crates/clankers) (currently v0.1.4). The items below track feature maturity inside the repo.



## v0.1.2 — Optimized inference

Shipped on crates.io as **v0.1.2**:

- [`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) as the primary optimized inference API (`builder`, `run_named`, `run_into`, `stats`)
- Zero-copy [`TensorView`](https://docs.rs/clankers-tensor/latest/clankers_tensor/struct.TensorView.html) inputs via `clankers-tensor`
- Modular [`InferenceEngine`](https://docs.rs/clankers-ml/latest/clankers_ml/inference/struct.InferenceEngine.html) + ONNX Runtime backend (power-user layer)
- `clankers bench` — latency percentiles and copy/allocation accounting
- ONNX fixture integration tests + template compile checks in CI
- `camera_replay` and `perception-node` template migrated to `run_named`

**Breaking changes from v0.1.1:** `Model::run` requires `&mut self`; `InferenceStats::copies` renamed to `clankers_copies`.

## v0.1.3 — Documentation polish

Shipped on crates.io as **v0.1.3**:

- Expanded [docs.rs](https://docs.rs/clankers) crate guides with quick-start examples on every crate

## v0.1.4 — C++ FFI + inference SDK (current)

Shipped on crates.io as **v0.1.4**, including the C/C++ inference bindings:

- Fixed crates.io README badges (shields.io version pins 404; use static or unversioned URLs)
- Added `clankers-cli` library target so docs.rs builds succeed
- **`clankers-ffi`** — stable C ABI (`clankers.h` via `cbindgen`) over `InferenceEngine`: engine builder, zero-copy `TensorView`, `run` / `run_named` / `run_into`, `InferenceStats`, panic-safe `extern "C"` entry points
- **`cpp/`** — C++17 RAII wrappers (`clankers::Engine`, `TensorView`, `Tensor`, `Error`), CMake packaging, `minimal_inference` and `zero_alloc_loop` examples
- **`scripts/build_cpp_sdk.sh`** — one-command `cargo build -p clankers-ffi` + CMake build
- **CI** — `cpp-sdk` job on Ubuntu 22.04 (FFI tests, C header `-Wall -Werror` smoke compile, example binaries); `no-default-features` covers noop-only `clankers-ffi`

**Not in v0.1.4 yet:** `find_package(clankers)` install docs, rclcpp perception colcon package (planned next).

## v1.0 — Production-ready SDK
- [x] Modular inference engine — backend-agnostic `InferenceEngine` over
  zero-copy `TensorView`s, with `InferenceBackend`/`BackendSession` traits, a
  `NoopBackend` and a refactored `OnnxRuntimeBackend` (zero-copy f32 input path),
  copy/allocation accounting (`InferenceStats`), preallocated-output `run_into`
  with a `Preallocate` arena for zero-alloc hot loops, `strict_realtime` build
  gating, ROS sensor adapters + composable pipeline transforms, `run_named`
  multi-input binding, [`Model`](https://docs.rs/clankers-ml/latest/clankers_ml/struct.Model.html) as the primary optimized inference API over the
  engine, and a
  `clankers bench` command reporting p50/p95/p99 latency and copies/allocations.
- [ ] Stable public APIs
- [~] Full rclrs integration — backend + typed `sensor_msgs/Image` bridge
  **compiled and run against ROS 2 Humble DDS** (verified 2026-07-07;
  `ros2 topic echo` sees a real `sensor_msgs/msg/Image`), now shipped as
  **checked-in colcon packages** ([`ros2/clankers-ros2-dds`](../ros2/clankers-ros2-dds)
  + [`ros2/pubsub_minimal_dds`](../ros2/pubsub_minimal_dds)) with a one-command
  build (`scripts/setup_ros2_ws.sh`) and a graceful executor shutdown. Remaining:
  a custom `.msg` for `DetectionArray` (currently on the `std_msgs/String` JSON
  path) and broader message coverage. See [docs/ros2_integration.md](ros2_integration.md).
- [~] C++ inference SDK — stable C ABI (`clankers-ffi`) + C++17 wrappers (`cpp/`)
  land in **v0.1.4** with ONNX `run`, `run_into`, and per-run stats. Remaining:
  rclcpp perception colcon package, `docs/cpp_integration.md`. See [cpp/README.md](../cpp/README.md).
- [ ] Optional LibTorch / ExecuTorch backends
- [ ] Expanded geometry and runtime modules
