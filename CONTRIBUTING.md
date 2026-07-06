# Contributing to clankeRS

Thank you for your interest in contributing!

## Development setup

```bash
git clone https://github.com/PvRao-29/clankeRS.git
cd clankeRS
cargo build --workspace
cargo test --workspace
```

For ROS 2 Humble integration, use the devcontainer or install ROS 2 on Ubuntu 22.04.

## Code style

- Run `cargo fmt` before committing
- Run `cargo clippy --workspace -- -D warnings`
- Keep changes focused and well-tested

## Pull requests

1. Fork [github.com/PvRao-29/clankeRS](https://github.com/PvRao-29/clankeRS)
2. Create a feature branch
3. Add tests for new behavior
4. Ensure CI passes
5. Open a PR with a clear description

## Crate structure

See [docs/architecture.md](docs/architecture.md).
