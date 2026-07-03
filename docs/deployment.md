# Deployment

## Build release binary

```bash
cargo build --release
```

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

1. `clankers validate-model` passed
2. Replay tests pass in CI
3. Latency within budget on target hardware
4. MCAP logging enabled for field debugging
