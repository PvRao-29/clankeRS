# Getting Started

## Install

```bash
cargo install --path crates/clankers-cli
```

Without installing, prefix commands with `cargo run -p clankers-cli --`:

```bash
cargo run -p clankers-cli -- new hello_robot --template basic-node
```

## Create your first node

```bash
clankers new hello_robot --template basic-node
cd hello_robot
clankers run
```

Expected output:

```text
clankeRS node started
Subscribed to /chatter
Publishing to /chatter
```

## Next steps

- Add an ONNX model: [PyTorch to ONNX](pytorch_to_onnx.md)
- Record and replay logs: [MCAP replay](mcap_replay.md)
- Write replay tests: [Testing](testing.md)
