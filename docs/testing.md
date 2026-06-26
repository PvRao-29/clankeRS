# Testing

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

## Run tests

```bash
clankers test
# or
cargo test
```

## CI

See `.github/workflows/ci.yml` for replay inspection in CI.
