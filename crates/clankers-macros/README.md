<p align="center">
  <strong>clankers-macros</strong><br>
  <em>Proc macros for clankeRS nodes and replay tests</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-macros"><img src="https://img.shields.io/crates/v/clankers-macros.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-macros"><img src="https://docs.rs/clankers-macros/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-macros.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Compile-time helpers that turn async node functions into `main` entry points and wire MCAP fixtures into `#[tokio::test]` replay tests. Re-exported from [`clankers`](https://crates.io/crates/clankers) as `clankers::node` and `clankers::replay_test`.

## Install

You typically depend on `clankers` rather than this crate directly:

```toml
clankers = "0.1"
```

Advanced users can depend on the macro crate alone:

```toml
clankers-macros = "0.1"
```

## `#[clankers::node]`

Wraps an async function as a robot node `main` — loads `clankeRS.toml`, initializes tracing, and runs tokio.

```rust
use clankers::prelude::*;

#[clankers::node]
async fn my_node(ctx: RobotContext) -> RobotResult<()> {
  tracing::info!(node = %ctx.node_name(), "starting");
  Ok(())
}
```

## `#[clankers::replay_test("path/to/log.mcap")]`

Runs your async test body against a replay context backed by an MCAP fixture.

```rust
use clankers::prelude::*;

#[clankers::replay_test("tests/fixtures/camera_log.mcap")]
async fn replay_is_clean(ctx: ReplayContext) -> RobotResult<()> {
    let result = ctx.run_replay(|_msg| async { Ok(()) }).await?;
    assert_no_panics(&result)?;
    Ok(())
}
```

## Learn more

- [Testing guide](https://github.com/PvRao-29/clankeRS/blob/main/docs/testing.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
