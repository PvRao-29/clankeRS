<p align="center">
  <strong>clankers-core</strong><br>
  <em>Core primitives for the clankeRS robotics SDK</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-core"><img src="https://img.shields.io/crates/v/clankers-core.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-core"><img src="https://docs.rs/clankers-core/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-core.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Foundational types, configuration, and error handling shared across every clankeRS crate. Most applications depend on the [`clankers`](https://crates.io/crates/clankers) facade instead of this crate directly.

## Install

```toml
clankers-core = "0.1"
```

```bash
cargo add clankers-core
# or, for the full SDK:
cargo add clankers
```

## Example

```rust
use clankers_core::{ClankeRSConfig, RobotContext, Timestamp, TopicName};

let cfg = ClankeRSConfig::load("clankeRS.toml")?;
let ctx = RobotContext::new(cfg, "/opt/robot");
let topic = TopicName::new("/camera/image_raw")?;
let now = Timestamp::now();
```

## Key types

| Type | Role |
|------|------|
| `ClankeRSConfig` | Parse `clankeRS.toml` project settings |
| `RobotContext` | Resolve paths, model configs, node name |
| `Timestamp`, `TopicName`, `NodeName`, `FrameId` | Strongly typed robotics primitives |
| `LatencyStats` | p50 / p95 / p99 latency reporting |
| `RobotError`, `RobotResult` | Unified error type |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [Architecture](https://github.com/PvRao-29/clankeRS/blob/main/docs/architecture.md)
- [Getting started](https://github.com/PvRao-29/clankeRS/blob/main/docs/getting_started.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
