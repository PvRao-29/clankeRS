# clankers-ffi

<p align="center">
  <strong>clankers-ffi</strong><br>
  <em>Stable C ABI for the clankeRS inference engine</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers-ffi"><img src="https://img.shields.io/static/v1?label=crates.io&message=v0.1.4&color=orange&style=flat-square" alt="crates.io"></a>
  <a href="https://docs.rs/clankers-ffi"><img src="https://docs.rs/clankers-ffi/badge.svg?style=flat-square" alt="docs.rs"></a>
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/LICENSE"><img src="https://img.shields.io/crates/l/clankers-ffi.svg?style=flat-square" alt="MIT license"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/clankers">clankeRS SDK</a> ·
  <a href="https://github.com/PvRao-29/clankeRS">GitHub</a> ·
  <a href="https://github.com/PvRao-29/clankeRS/blob/main/cpp/README.md">C++ SDK</a>
</p>

---

Stable C ABI for the clankeRS inference engine (v0.1.4). C++ and other language bindings
link this crate (`cdylib` / `staticlib`) rather than calling Rust APIs directly.

## Install

```toml
clankers-ffi = "0.1"
```

```bash
cargo add clankers-ffi
```

Build the native library:

```bash
cargo build -p clankers-ffi --release
```

The generated header is written to `include/clankers/clankers.h` inside this crate
during the build (and synced into `cpp/include/clankers/` when built from the clankeRS
tree). See [`cpp/README.md`](https://github.com/PvRao-29/clankeRS/blob/main/cpp/README.md) for the idiomatic C++17 wrappers.
