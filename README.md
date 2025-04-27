# MC Server

*A small experiment to mimic a Minecraft server*

Provides a demo that renders an analog clock using phantoms. Only supports vanilla 1.21.4 (Minecraft is very picky about protocol versions).

## Usage

Ensure that Java is installed and in path. Then run:

```sh
./generate_registries.sh # Required to extract block state and entity IDs from the official binaries
cargo run
```

Then launch Minecraft 1.21.4 (vanilla) and add `localhost` as a server.
