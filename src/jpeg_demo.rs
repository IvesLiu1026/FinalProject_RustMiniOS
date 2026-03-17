pub struct EmbeddedJpegDemo {
    pub label: &'static str,
    pub data: &'static [u8],
}

const DUNGEON_PREVIEW_JPEG: &[u8] =
    include_bytes!("../assets/test_media/images/dungeon_preview.jpeg");
const CREATURES_PREVIEW_JPEG: &[u8] =
    include_bytes!("../assets/test_media/images/creatures_preview.jpeg");

pub static JPEG_DEMOS: [EmbeddedJpegDemo; 2] = [
    EmbeddedJpegDemo {
        label: "Dungeon",
        data: DUNGEON_PREVIEW_JPEG,
    },
    EmbeddedJpegDemo {
        label: "Creatures",
        data: CREATURES_PREVIEW_JPEG,
    },
];

pub fn demos() -> &'static [EmbeddedJpegDemo] {
    &JPEG_DEMOS
}
