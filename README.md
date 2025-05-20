# MC Server

*A small experiment to mimic a Minecraft server*

Provides a demo that renders an analog clock using phantoms. Only supports vanilla 1.21.4 (Minecraft is very picky about protocol versions).

## Usage

Ensure that [Java](https://formulae.brew.sh/formula/openjdk) and [Rust](https://rustup.rs) are installed and in the path. Then run:

```sh
./generate_registries.sh # Required to extract block state and entity IDs from the official binaries
cargo run --bin xray
```

Then launch Minecraft 1.21.4 (vanilla) and add `localhost` as a server.
