<p align="center">
  <strong>clankers-data</strong><br>
  <em>MCAP logging, replay, and inspection for clankeRS</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-data"><img src="https://img.shields.io/crates/v/clankers-data.svg?style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-data"><img src="https://docs.rs/clankers-data/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-data.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a>
</p>

---

Read, write, replay, and compare [MCAP](https://mcap.dev/) robot logs — the backbone of clankeRS replay testing and offline debugging.

## Install

```toml
clankers-data = "0.1"
```

```bash
cargo add clankers-data
```

## Example

```rust
use clankers_data::{format_inspect_report, McapLog, Replay};

// Summarize topics and message counts
let log = McapLog::open("logs/run.mcap")?;
println!("{}", format_inspect_report(log.report()));

// Replay every message through your handler
let replay = Replay::from_mcap("logs/run.mcap")?;
let result = replay
    .run(|msg| async move {
        println!("{}: {} bytes", msg.topic, msg.data.len());
        Ok(())
    })
    .await?;
println!("replayed {} messages", result.summary.input_messages);
```

## Key types

| Type | Role |
|------|------|
| `Replay` | Async replay over an MCAP file |
| `McapWriter` | Write timestamped messages to MCAP |
| `McapLog`, `InspectReport` | Open a log and print a human summary |
| `compare_logs` | Diff two MCAP files for regression checks |

## Learn more

- [MCAP replay guide](https://github.com/PvRao-29/clankeRS/blob/main/docs/mcap_replay.md)
- [Testing with replay](https://github.com/PvRao-29/clankeRS/blob/main/docs/testing.md)

## License

MIT — see [LICENSE](https://github.com/PvRao-29/clankeRS/blob/main/LICENSE).
