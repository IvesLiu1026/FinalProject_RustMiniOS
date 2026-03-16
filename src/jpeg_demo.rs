pub struct EmbeddedJpegDemo {
    pub label: &'static str,
    pub data: &'static [u8],
}

const CHERRY_JPEG: &[u8] = include_bytes!("../assets/test_media/images/cherry.jpeg");
const RUUU_JPEG: &[u8] = include_bytes!("../assets/test_media/images/ruuu.jpeg");

pub static JPEG_DEMOS: [EmbeddedJpegDemo; 2] = [
    EmbeddedJpegDemo {
        label: "Cherry",
        data: CHERRY_JPEG,
    },
    EmbeddedJpegDemo {
        label: "Ruuu",
        data: RUUU_JPEG,
    },
];

pub fn demos() -> &'static [EmbeddedJpegDemo] {
    &JPEG_DEMOS
}
