# platec-rs

`platec-rs` is the Rust port used by Uncivilized (and some other my in-dev games, for instance - TheiaRPG) for tectonic terrain generation.

It is derived from [Mindwerks Plate Tectonics](https://github.com/Mindwerks/plate-tectonics), itself a fork of platec by Lauri Viitanen. The port and its modifications are available under LGPL-2.1-or-later; see `THIRD_PARTY_NOTICES.md`.

## Dynamic-library build

Build the library independently from the game:

```text
cargo build --release
```

On Windows this creates `target/release/platec_rs.dll`. Uncivilized loads that exact release DLL only while generating a map; its own Cargo build never rebuilds this crate.

## C ABI

`platec_generate_map_once` is the sole generation entrypoint. It receives a seed and dimensions, runs the complete simulation internally, then fills caller-owned height and relief arrays. The caller owns all output memory, so it may unload the DLL immediately after the function returns.

## How it looks
For example - that it is looks like in TheiaRPG map generator (closed source now, and climate model is closed, but heightmap is the same):
<img width="1895" height="866" alt="изображение" src="https://github.com/user-attachments/assets/991d9c37-9281-4ad0-923f-16bdad992df4" />
<img width="1920" height="1079" alt="изображение" src="https://github.com/user-attachments/assets/c74a1153-4c96-42a9-9740-9665594f9439" />
