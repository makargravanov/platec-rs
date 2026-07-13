# platec-rs

`platec-rs` is the Rust port used by Uncivilized for tectonic terrain generation.

It is derived from [Mindwerks Plate Tectonics](https://github.com/Mindwerks/plate-tectonics), itself a fork of platec by Lauri Viitanen. The port and its modifications are available under LGPL-2.1-or-later; see `THIRD_PARTY_NOTICES.md`.

## Dynamic-library build

Build the library independently from the game:

```text
cargo build --release
```

On Windows this creates `target/release/platec_rs.dll`. Uncivilized loads that exact release DLL only while generating a map; its own Cargo build never rebuilds this crate.

## C ABI

`platec_generate_map_once` is the sole generation entrypoint. It receives a seed and dimensions, runs the complete simulation internally, then fills caller-owned height and relief arrays. The caller owns all output memory, so it may unload the DLL immediately after the function returns.
