# Mac Companion

`Album` currently defaults to embedded media, and `Mac companion` is kept as an optional lower-flash build path.

## What changed

- firmware can skip embedded Album stills and motion clips when you choose the companion build
- Album now prefers a runtime media source over `USART3`
- the Mac host serves converted `RGB565` stills and motion frames on demand
- if you build the companion version and no host is connected, Album shows a connection hint

This keeps a future low-flash option available while preserving the simpler embedded Album flow as the default.

## Wiring

Board side:

- `USART3 TX`: `PC10`
- `USART3 RX`: `PC11`
- baud rate: `921600`

Mac side:

- connect through a USB-UART adapter
- adapter `TX` goes to board `PC11`
- adapter `RX` goes to board `PC10`
- connect `GND` between Mac adapter and the board

If your board only exposes ST-Link for flashing, you will usually still need a separate USB-UART bridge for the companion link.

## Media source

The companion serves the already-processed media inside the repo:

- stills: [assets/test_media/converted/firmware/stills](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/assets/test_media/converted/firmware/stills)
- motion: [assets/test_media/converted/firmware/motion](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/assets/test_media/converted/firmware/motion)
- manifests: [assets/test_media/converted/manifests](/Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/assets/test_media/converted/manifests)

If you add or replace images/GIFs, run:

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS
./tools/process_test_media.sh
```

## Running the host

List ports first if needed:

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/mac_companion
cargo run -- --list-ports
```

Start the companion:

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/mac_companion
cargo run -- --port /dev/tty.usbserial-XXXX
```

Optional flags:

```bash
--baud 921600
--media-root /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS/assets/test_media/converted
```

## Firmware build behavior

The default firmware build keeps the embedded Album path.

To switch to the companion build:

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS
MINIOS_EMBED_ALBUM=0 cargo build --release
```

To force the regular embedded Album build:

```bash
MINIOS_EMBED_ALBUM=1 cargo build --release
```

## Validation checklist

1. Flash the current firmware to the board.
2. Start the Mac companion on the correct serial port.
3. Open `Album` on the board.
4. Confirm the source chip changes to `MAC LINK`.
5. Browse stills with `K0` / `WKUP`.
6. Switch to motion clips with `K1`.
7. Confirm motion frames continue to advance while the companion is connected.

## Current scope

This first version is intentionally simple:

- single host
- serial request / response protocol
- one runtime frame buffer on the MCU
- converted `RGB565` assets only

That keeps it understandable and matches the board's resource limits much better than embedding a large media library into flash.
