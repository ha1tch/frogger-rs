# Frogger-RS

A Frogger clone written in Rust using raylib.

## Overview

Classic arcade-style Frogger gameplay:
- Guide your frog from the bottom of the screen to the lily pads at the top
- Avoid cars and trucks on the road
- Hop onto floating logs to cross the river (don't fall in!)
- Fill all 5 lily pad slots to win

## Controls

| Key | Action |
|-----|--------|
| W / ↑ | Move up |
| S / ↓ | Move down |
| A / ← | Move left |
| D / → | Move right |
| Space | Restart (when game over) |
| Esc | Quit |

## Building

Requires Rust and system dependencies for raylib.

### Linux (Debian/Ubuntu)

```bash
# Install raylib dependencies
sudo apt-get install libasound2-dev libx11-dev libxrandr-dev libxi-dev \
    libgl1-mesa-dev libglu1-mesa-dev libxcursor-dev libxinerama-dev

# Build and run
cargo run --release
```

### macOS

```bash
brew install raylib
cargo run --release
```

### Windows

The raylib crate will download pre-built binaries automatically.

```bash
cargo run --release
```

## Game Layout

```
Row 0:      [Goal] Lily pads - reach these to score
Rows 1-5:  [River] Floating logs - ride them or drown
Row 6:      [Safe] Rest area
Rows 7-12: [Road] Cars and trucks - instant death on contact
Row 13:    [Safe] Rest area
Row 14:    [Start] Frog starting position
```

## Features

- 3 lives
- Score tracking (100 points per goal reached)
- Logs move with the current, carrying the frog along
- Randomised vehicle and log placement each game
- Win/lose screens with restart option

## Project Structure

```
frogger-rs/
├── Cargo.toml
├── README.md
└── src/
    └── main.rs
```

## License

Copyright (C) 2025 ha1tch

Apache 2.0

## Author

ha1tch

h@ual.fi

https://oldbytes.space/user/@haitchfive


