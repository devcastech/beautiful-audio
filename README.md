![metronome](https://github.com/user-attachments/assets/d688cccb-83a5-4371-aa25-b3af1dbf8c9d)

# BE Labs Audio Metronome

A metronome written in Rust, built as a standalone app with [nice-plug](https://crates.io/crates/nice-plug) and an [iced](https://iced.rs) editor.

- `core/`: the metronome engine, with no dependencies
- `metronome/`: the app and its editor

## Run

```sh
cargo run --release -p metronome
```

## Test

```sh
cargo test -p audio-core
cargo run -p audio-core --example render_wav   # writes metronome.wav
```
