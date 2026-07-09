<p align="center">
  <strong>clankers-testing</strong><br>
  <em>Replay-based testing tools for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-testing"><img src="https://img.shields.io/static/v1?label=crates.io&message=v0.1.4&color=orange&style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-testing"><img src="https://docs.rs/clankers-testing/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-testing.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/docs/testing.md">Testing guide</a>
</p>

---

Run your node logic against recorded MCAP fixtures and assert on topics, latency, and dropped messages — regression tests grounded in real robot logs.

## Install

```toml
clankers-testing = "0.1"
```

```bash
cargo add clankers-testing
```

Or use the `#[clankers::replay_test]` macro from the [`clankers`](https://crates.io/crates/clankers) facade.

## Example

```rust
use std::time::Duration;
use clankers_testing::{
    assert_max_latency, assert_no_panics, assert_topic_exists, ReplayContext,
};

#[tokio::test]
async fn camera_log_replays_cleanly() {
    let ctx = ReplayContext::new("tests/fixtures/camera_log.mcap");
    let result = ctx.run_replay(|_msg| async { Ok(()) }).await.unwrap();

    assert_no_panics(&result).unwrap();
    assert_topic_exists(&result, "/camera/image_raw").unwrap();
    assert_max_latency(&result, Duration::from_secs(10)).unwrap();
}
```

## Assertions

| Function | Checks |
|----------|--------|
| `assert_topic_exists` | A topic appeared during replay |
| `assert_no_panics` | Handler completed without panics |
| `assert_dropped_messages` | Drop count within budget |
| `assert_max_latency` | p99 latency under a ceiling |

## Learn more

- [Installation](https://github.com/PvRao-29/clankeRS/blob/main/docs/installation.md)
- [MCAP replay](https://github.com/PvRao-29/clankeRS/blob/main/docs/mcap_replay.md)
- [Testing guide](https://github.com/PvRao-29/clankeRS/blob/main/docs/testing.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
