# FinalProject_RustMiniOS

`FinalProject_RustMiniOS` is a Rust bare-metal prototype for the shared `STM32F407ZG + 320x240 ILI9341 TFT resistive touch` board.

This repository is intentionally separate from:

- `/Users/ivesliu/Documents/MCP2026/FinalProject_MiniOS`

## What it does

The current prototype includes:

- animated boot splash
- mandatory touch calibration flow on startup
- touch-first home launcher
- map selection
- settings for theme, language, and render strategy
- a control/status page for board interaction
- a textured 3D dungeon prototype built with software raycasting
- enemy movement and damage
- weapon switching
- pickups and healing
- victory / defeat overlays
- FPS counter

## Project structure

- `src/main.rs`
  - top-level MiniOS flow, screen switching, touch calibration state
- `src/ui.rs`
  - system UI screens such as Home, Map Select, Settings, Touch Calibration
- `src/dungeon.rs`
  - dungeon gameplay, rendering, HUD, touch controls
- `src/dungeon/data.rs`
  - map layouts, enemy spawns, pickup spawns
- `src/dungeon/weapon.rs`
  - weapon definitions and tuning
- `src/dungeon/strategy.rs`
  - render quality modes
- `src/touch.rs`
  - resistive touch sampling and calibration
- `src/display.rs`
  - Rust-side display helpers and framebuffer upload glue
- `c_support/`
  - reused STM32 clock init and TFT driver code compiled through FFI
- `assets/`
  - curated art assets and source packs
- `preview/`
  - lightweight browser UI preview used during iteration
- `tools/`
  - helper scripts such as font generation and linker wrapper

## Controls

### Startup

- system boots into `Touch Calibration`
- tap the five calibration targets in order
- after calibration, the system enters `Home`

### Home

- `K0`: previous card
- `WKUP`: next card
- `K1`: open selected card
- touch: tap a card directly

### Settings

- theme toggle
- English / Traditional Chinese toggle
- render strategy cycle:
  - `QUALITY`
  - `BALANCED`
  - `PERFORMANCE`

### Dungeon Core

- left virtual joystick: move / turn
- right virtual button: fire
- `WKUP + K1`: next weapon
- `K0 + K1`: previous weapon
- tap the center weapon chip: cycle weapon
- hold `K0 + WKUP`: return

## Render strategy

The prototype supports three rendering profiles:

- `QUALITY`
  - full floor / ceiling detail
- `BALANCED`
  - reduced floor / ceiling cost with similar overall look
- `PERFORMANCE`
  - lower-cost wall and floor rendering for higher FPS

## Build

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS
cargo build --release
```

## Flash

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS
cargo run --release
```

## Environment notes

- Rust target: `thumbv7em-none-eabihf`
- MCU: `STM32F407ZG`
- LCD path: FSMC / 8080-style parallel bus
- the project expects PlatformIO packages under the default `~/.platformio/packages`
- if your PlatformIO packages live somewhere else, set:

```bash
export PLATFORMIO_PACKAGES_DIR=/your/path/to/.platformio/packages
```

## Notes

- touch is single-point resistive touch, not multi-touch
- Chinese glyph data is generated into `src/font_zh.rs`
- this repository does not modify `/Users/ivesliu/Documents/MCP2026/FinalProject_MiniOS`
