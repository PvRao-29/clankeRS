# Deployment

## Build release binary

From your clankeRS project (created with `clankers new` or `cargo add clankers`):

```bash
cargo build --release
```

The binary is at `target/release/<package-name>`.

## Docker (robot computer)

```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y ca-certificates
COPY target/release/my_node /usr/local/bin/
COPY models/ /models/
COPY clankeRS.toml /
CMD ["my_node"]
```

## Cross-compilation

For ARM robot computers, use `cross` or a target-specific toolchain. ONNX Runtime binaries must match the target architecture.

## Checklist

1. `clankers validate-model` passed on representative inputs
2. Replay tests pass in CI
3. Latency within budget on target hardware (`clankers bench` for p50/p95/p99 + copy stats)
4. MCAP logging enabled for field debugging (when `record_mcap` support lands)

## Related docs

- [Model validation](model_validation.md)
- [Testing](testing.md)
- [Installation](installation.md) — crates.io install for new projects
