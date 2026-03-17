# MiniOS Firmware Bundle

This bundle is meant for anyone who has the same `STM32F407ZG + ST-Link + ILI9341 + resistive touch` hardware.

## What You Need

- the matching board and display wiring
- a USB connection to the board's ST-Link
- one flashing tool:
  - `probe-rs` on macOS / Linux / Windows
  - or `openocd` on macOS / Linux

Official probe-rs installation docs:
- https://probe.rs/docs/getting-started/installation
- https://probe.rs/docs/getting-started/probe-setup

Common install options from the official docs:

```bash
cargo install probe-rs-tools --locked
```

## Quick Flash

### macOS / Linux

```bash
./flash.sh
```

By default this uses `probe-rs` if it is installed, otherwise it falls back to `openocd`.

The bundled `probe-rs` path follows the official `probe-rs run --chip <chip-name> <firmware.elf>` workflow.

Force a tool explicitly:

```bash
MINIOS_FLASH_TOOL=probe-rs ./flash.sh
MINIOS_FLASH_TOOL=openocd ./flash.sh
```

### Windows

```powershell
.\tools\flash-windows.ps1
```

## Firmware Files

- `firmware/finalproject_rustminios.elf`
- `firmware/finalproject_rustminios.bin`
- `firmware/finalproject_rustminios.hex`

Use the `.elf` with the bundled flash scripts. The `.bin` and `.hex` are included for external tooling and lab workflows.

## Notes

- default probe-rs chip name: `STM32F407ZGTx`
- override it if needed with `MINIOS_PROBE_RS_CHIP`
- if your OpenOCD install lives in a custom place, set `MINIOS_OPENOCD` or `OPENOCD_SCRIPTS`
- on Windows, some probes need a WinUSB driver; see the official probe setup guide above
