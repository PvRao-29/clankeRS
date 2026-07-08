# Testing

Replay-based tests let you validate node behavior against recorded MCAP logs.

## Prerequisites

Add clankeRS to your project:

```bash
cargo add clankers
```

Optional CLI for `clankers test`:

```bash
cargo install clankers-cli
```

Scaffold a project with replay tests built in:

```bash
clankers new my_robot --template replay-test-node
```

## Replay-based tests

```rust
use clankers::prelude::*;

#[clankers::replay_test("tests/data/camera_sample.mcap")]
async fn camera_detector_replay_test(ctx: ReplayContext) -> Result<()> {
    let result = ctx
        .run_replay(|_msg| async { Ok(()) })
        .await?;

    assert_topic_exists(&result, "/camera/image_raw")?;
    assert_no_panics(&result)?;
    assert_max_latency(&result, Duration::from_millis(20))?;
    assert_dropped_messages(&result, 0)?;

    Ok(())
}
```

Place your MCAP fixture under `tests/data/` in your project. Bundled sample logs are in the repo under `sample_data/` — see [Installation](installation.md#clone-for-development-or-the-bundled-demo).

## Run tests

```bash
clankers test
# or
cargo test
```

## CI

See `.github/workflows/ci.yml` for replay inspection in CI.
