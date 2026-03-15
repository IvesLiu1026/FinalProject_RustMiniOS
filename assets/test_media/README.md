# Test Media Inbox

Place temporary media here for Codex-side conversion and playback testing.

Folders:

- `images/`
  - Drop still images here, such as `.png`, `.jpg`, `.jpeg`, or `.bmp`.
- `gifs/`
  - Drop short animated GIF files here for frame extraction tests.
- `converted/`
  - Generated outputs from `tools/process_test_media.sh`.

Recommended file constraints for early tests:

- Images:
  - Prefer `320x240` or smaller.
  - If larger, keep them under about `800x600` for easier preprocessing.
- GIFs:
  - Prefer short loops, around `2-5` seconds.
  - Prefer low frame counts.
  - Avoid very large files for the first pass.

Suggested naming:

- `images/album_01.png`
- `images/album_02.jpg`
- `gifs/boot_loop_01.gif`
- `gifs/pet_idle_01.gif`

Processing command:

```bash
cd /Users/ivesliu/Documents/MCP2026/FinalProject_RustMiniOS
./tools/process_test_media.sh
```

What the script does:

- Converts still images into `320x240` preview PNG files.
- Preserves the whole source image with a `contain` fit.
- Fills unused space with a blurred background derived from the same image.
- Converts GIFs into processed `320x240` frame sequences plus preview GIFs.
- Emits low-resolution `RGB565` firmware assets for the embedded album and motion player.
- Drops simple manifest files for later firmware or host-tool integration.
