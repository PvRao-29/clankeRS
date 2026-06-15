# MCAP Replay

## Inspect a log

```bash
clankers inspect sample_data/camera_log.mcap
```

## Replay

```bash
clankers replay sample_data/camera_log.mcap
```

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
