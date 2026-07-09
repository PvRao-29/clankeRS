<p align="center">
  <strong>clankers-runtime</strong><br>
  <em>Execution, scheduling, and observability for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-runtime"><img src="https://img.shields.io/crates/v/clankers-runtime/0.1.3.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-runtime"><img src="https://docs.rs/clankers-runtime/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-runtime.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Configure queue depth, deadlines, and runtime metrics for clankeRS nodes. Also provides tracing initialization used by the `#[clankers::node]` macro.

## Install

```toml
clankers-runtime = "0.1"
```

```bash
cargo add clankers-runtime
```

## Example

```rust
use std::time::Duration;
use clankers_runtime::RobotRuntime;

let rt = RobotRuntime::builder()
    .max_queue_depth(8)
    .deadline(Duration::from_millis(33)) // ~30 Hz
    .enable_latency_tracing(true)
    .build()?;

println!("{}", rt.metrics().format_report());
```

Initialize structured logging in a binary:

```rust
clankers_runtime::runtime::init_tracing();
```

## Key types

| Type | Role |
|------|------|
| `RobotRuntime` | Queue depth, deadline, and metrics config |
| `RuntimeMetrics` | Counters and latency summaries at runtime |
| `init_tracing` | `tracing-subscriber` setup for nodes |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Architecture](https://github.com/PvRao-29/clankeRS/blob/main/docs/architecture.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
