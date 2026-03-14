# FinalProject_RustMiniOS Assets

This folder contains legally usable art packs for the Rust mini-OS and dungeon game.

## Structure

- `source_packs/`
  - Original downloaded zip files.
- `raw/`
  - Unpacked original contents from each source pack.
- `curated/`
  - A smaller starter set chosen for the current raycaster and UI prototype.

## Current Packs

### 1. Old School Dungeon Crawler Pack

- Source page: `https://opengameart.org/content/old-school-dungeon-crawler-pack`
- License: `CC0`
- Best use:
  - First-person wall textures
  - Doors
  - Window variants

Downloaded archives:

- `old_school_dungeon_crawler_walls.zip`
- `old_school_dungeon_crawler_windows.zip`

Starter textures copied into `curated/raycaster_textures/`:

- `wall_brick_light.png`
- `wall_brick_mid.png`
- `wall_brick_dark.png`
- `door_brick_light.png`
- `door_brick_mid.png`
- `door_brick_dark.png`
- `window_brick_light.png`
- `window_brick_mid.png`
- `window_brick_dark.png`

### 2. Tiny Dungeon

- Source page: `https://opengameart.org/content/tiny-dungeon`
- License: `CC0`
- Best use:
  - Minimap visuals
  - Decorative UI art
  - Top-down room layout experiments

Useful files:

- `raw/kenney_tiny_dungeon/Tilemap/tilemap_packed.png`
- `curated/tilemaps/kenney_tiny_dungeon_tilemap.png`
- `curated/previews/kenney_tiny_dungeon_preview.png`

### 3. Tiny Creatures

- Source page: `https://opengameart.org/content/tiny-creatures`
- License: `CC0`
- Best use:
  - Enemy sprite exploration
  - NPC and creature selection
  - Later bestiary or combat prototypes

Useful files:

- `raw/tiny_creatures/tiny-creatures/Tilemap/tilemap_packed.png`
- `curated/tilemaps/tiny_creatures_tilemap.png`
- `curated/previews/tiny_creatures_preview.png`

## Recommended First Pass

For the current dungeon prototype, start with:

- `curated/raycaster_textures/wall_brick_dark.png`
- `curated/raycaster_textures/door_brick_dark.png`
- `curated/raycaster_textures/window_brick_dark.png`

This gives the game a darker, more cohesive dungeon mood immediately.

Then use:

- `curated/tilemaps/tiny_creatures_tilemap.png`

to choose 1-2 enemies for the first encounter pass.

## Suggested Next Step

Convert the selected PNGs into a Rust-friendly runtime format, likely:

- `RGB565`
- fixed-size texture arrays
- optional transparency key for sprites

The next implementation step should focus on:

1. one wall texture path in the raycaster
2. one door texture
3. one enemy sprite billboard
