# Flash Usage Analysis

- Timestamp: `2026-03-16 02:56:18 CST`
- Project: `FinalProject_RustMiniOS`
- Target board: `STM32F407ZG`

## Flash map

The linker keeps the top `128 KiB` of the MCU flash for persistent storage, so the firmware image does not get the full `1 MiB`.

From [memory.x](./memory.x):

- app flash region: `896 KiB` (`917,504 bytes`)
- storage region: `128 KiB` (`131,072 bytes`)
- main SRAM: `128 KiB`
- CCM RAM: `64 KiB` for stack isolation

For flash budgeting, the relevant sections are:

- `.vector_table`
- `.text`
- `.rodata`
- `.data`

Debug sections such as `.debug_info` and `.debug_line` are not programmed into the MCU flash.

## Baseline before Mac companion

Measurement taken from the release image before removing embedded Album media:

```text
text = 518,652
data = 16
bss  = 32,724
```

Section breakdown:

```text
.vector_table =     392
.text         = 158,524
.rodata       = 359,728
.data         =      16
```

Effective programmed flash footprint:

```text
392 + 158,524 + 359,728 + 16 + 4(.init) + 4(.fini)
= 518,668 bytes
```

Headroom inside the `896 KiB` app region:

```text
917,504 - 518,668 = 398,836 bytes
```

## Why the baseline was large

The dominant flash consumer was not pure code. It was embedded media.

At the time of measurement, the asset library on disk was:

- Album stills: `86,400 bytes`
- Album motion frames: `211,200 bytes`
- Dungeon textures: `40,960 bytes`
- Dungeon sprite RGB565 data: `3,456 bytes`

That means the Album media alone accounted for:

```text
86,400 + 211,200 = 297,600 bytes
```

This lines up closely with the old `.rodata` size of `359,728 bytes`, which is why Album media was the first place to optimize.

## After switching Album to Mac companion

The current default build no longer embeds Album stills or motion clips into firmware. They are served from the Mac companion instead.

Current release measurement:

```text
text = 231,440
data = 16
bss  = 55,160
```

Section breakdown:

```text
.vector_table =     392
.text         = 168,388
.rodata       =  62,652
.data         =      16
```

Effective programmed flash footprint:

```text
392 + 168,388 + 62,652 + 16 + 4(.init) + 4(.fini)
= 231,456 bytes
```

Headroom inside the `896 KiB` app region:

```text
917,504 - 231,456 = 686,048 bytes
```

## Delta

Flash reduction from the Mac companion change:

```text
518,668 - 231,456 = 287,212 bytes saved
```

That is about `55.4%` less programmed flash than the embedded-Album baseline.

The tradeoff is RAM:

- old `.bss`: `32,724`
- new `.bss`: `55,160`
- increase: `22,436`

This increase is expected because the board now keeps a runtime companion frame buffer in RAM instead of storing Album media in flash. The system still remains comfortably inside the `128 KiB` main SRAM budget, and the stack is already isolated in CCM RAM.

## What currently affects flash the most

1. Embedded media and texture assets in `.rodata`
2. Large render paths in `.text`, especially shell rendering, dungeon rendering, and display text drawing
3. Math support pulled in by floating-point heavy code paths

Even after the Album move, these still matter:

- dungeon texture data stays embedded
- fonts remain embedded
- large rendering functions still dominate `.text`

## Most effective optimization order

1. Move large media libraries out of firmware flash
2. Keep motion clips off-board or heavily preprocessed
3. Reuse textures and fonts rather than adding many new embedded assets
4. Continue splitting and simplifying large render functions
5. Audit accidental heavy math usage if code size becomes the next bottleneck

## Practical takeaway

For this repo, the best flash optimization was not micro-tuning codegen. It was architectural:

- keep persistent storage in a reserved flash sector
- keep stack out of main SRAM with CCM RAM
- keep large Album media on the Mac side instead of in firmware

That change alone turned flash from a future concern into a comfortable budget again.
