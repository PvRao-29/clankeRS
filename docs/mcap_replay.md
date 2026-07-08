# MCAP Replay

Inspect, replay, and compare robot logs with the `clankers` CLI.

## Prerequisites

```bash
cargo install clankers-cli
```

The examples below use `sample_data/camera_log.mcap` from the [GitHub repository](installation.md#clone-for-development-or-the-bundled-demo). Replace paths with your own MCAP files when working outside a clone.

## Inspect a log

```bash
clankers inspect sample_data/camera_log.mcap
```

## Replay

```bash
clankers replay sample_data/camera_log.mcap
```

Replays messages and reports stats. To drive your node through a log, pass `--node` (see `clankers replay --help`).

## Latency report

```bash
clankers latency sample_data/camera_log.mcap
```

## Compare expected vs actual

```bash
clankers compare expected.mcap actual.mcap
```

## Record during node execution

Enable in `clankeRS.toml`:

```toml
[logging]
record_mcap = true
output_dir = "logs/"
```

Then run `clankers run`.

> `clankers record` is a stub in v0.1 — MCAP recording from `clankers run` is not complete yet.
