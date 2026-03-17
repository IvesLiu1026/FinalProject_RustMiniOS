# FinalProject_RustMiniOS

Bare-metal Rust MiniOS for the shared `STM32F407ZG + 320x240 ILI9341 resistive-touch TFT` board.

This project turns the board into a small touch-first desktop shell with multiple apps, arcade-style games, diagnostics, persistence, and a portable firmware release flow.

## Overview

`FinalProject_RustMiniOS` is built around a lightweight embedded shell that boots into a launcher and hosts:

- `Album` with embedded media and an optional Mac companion mode
- `Game Center` with `Station Hunter`, `Tap Rush`, `Pseudo Racer`, `Graphics Lab`, and `Dungeon Core`
- `Pixel Paint`
- `Settings`, `Diagnostics`, `About`, `Benchmark`, and touch calibration tools

The repo is meant to show that a single STM32 board can run a polished, real-time Rust UI/game system instead of just isolated demos.

## Highlights

- desktop-style MiniOS shell with bilingual UI and theme switching
- persistent settings and app save data stored in a reserved flash sector
- optional `Mac companion` media pipeline over `USART3`
- multiple game loops on real hardware, including 2D action and 3D raycasting
- benchmark / performance console for board-side diagnostics
- host-side checks plus a portable firmware bundle for same-board deployment

## Hardware

| Item | Value |
| --- | --- |
| MCU | STM32F407ZG |
| Display | 320x240 ILI9341 |
| LCD bus | FSMC / 8080-style parallel |
| Touch | Single-point resistive touch |
| Rust target | `thumbv7em-none-eabihf` |

## Repository Layout

- `src/`: firmware source, MiniOS shell, app modules, storage, rendering, and board integration
- `c_support/`: reused STM32 clock / TFT support code through FFI
- `assets/`: textures, converted media, and reference assets
- `tools/`: preprocessing, flashing, bundle, and toolchain helper scripts
- `host_checks/`: host-side tests for storage, registry, and media invariants
- `mac_companion/`: optional desktop serial host for Album media streaming
- `docs/`: focused user/developer notes such as the smoke checklist and Mac companion setup

## Build

```bash
cargo build --release
```

## Flash

```bash
cargo run --release
```

`cargo run --release` uses [`flash.sh`](flash.sh), which prefers `probe-rs` when available and falls back to `openocd`.

Force a flashing backend explicitly:

```bash
MINIOS_FLASH_TOOL=probe-rs cargo run --release
MINIOS_FLASH_TOOL=openocd cargo run --release
```

## Portable Firmware Bundle

To create a ready-to-share firmware package for anyone with the same board:

```bash
./tools/make-release-bundle.sh
```

This produces:

- `dist/finalproject_rustminios-v0.1.0-stm32f407zg/`
- `dist/finalproject_rustminios-v0.1.0-stm32f407zg.zip`

The bundle includes `.elf`, `.bin`, `.hex`, flash scripts, and `QUICKSTART.md`.

## Media Pipeline

Test media lives under:

- `assets/test_media/images`
- `assets/test_media/gifs`

Run:

```bash
./tools/process_test_media.sh
```

The default firmware build embeds Album stills and motion clips. To build the lighter companion mode instead:

```bash
MINIOS_EMBED_ALBUM=0 cargo build --release
```

More details: [docs/mac-companion.md](docs/mac-companion.md)

## Verification

Run the host-side checks with:

```bash
(cd host_checks && cargo test)
```

Useful runtime notes:

- [docs/smoke-test-checklist.md](docs/smoke-test-checklist.md)
- [docs/mac-companion.md](docs/mac-companion.md)
- [docs/repo-architecture.md](docs/repo-architecture.md)

## Environment Notes

The helper scripts first try system tools on `PATH`, then fall back to PlatformIO-managed packages under `~/.platformio/packages`.

Override tool locations manually if needed:

```bash
export PLATFORMIO_PACKAGES_DIR=/your/path/to/.platformio/packages
export MINIOS_ARM_GCC=/path/to/arm-none-eabi-gcc
export MINIOS_ARM_OBJCOPY=/path/to/arm-none-eabi-objcopy
export MINIOS_ARM_SIZE=/path/to/arm-none-eabi-size
export MINIOS_ARM_NM=/path/to/arm-none-eabi-nm
export MINIOS_OPENOCD=/path/to/openocd
export OPENOCD_SCRIPTS=/path/to/openocd/scripts
```

Official `probe-rs` docs:

- [Installation](https://probe.rs/docs/getting-started/installation)
- [Probe Setup](https://probe.rs/docs/getting-started/probe-setup)

## Notes

- `touch` is resistive single-touch, not true multi-touch
- Chinese glyph data is generated into `src/font_zh.rs`
- build-time media registration is generated from `assets/test_media/converted`
- this repository is intentionally separate from the legacy C-based `FinalProject_MiniOS`
