use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("missing manifest dir"));
    let project_root = manifest_dir.as_path();
    let platformio_home = platformio_packages_dir();
    let gcc = platformio_home.join("toolchain-gccarmnoneeabi/bin/arm-none-eabi-gcc");
    let ar = platformio_home.join("toolchain-gccarmnoneeabi/bin/arm-none-eabi-ar");
    let cmsis_core = platformio_home.join("framework-cmsis/CMSIS/Core/Include");
    let cmsis_device = platformio_home.join("framework-cmsis-stm32f4/Include");

    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rerun-if-changed=c_support/clock/hardware_system_init.c");
    println!("cargo:rerun-if-changed=c_support/tft");
    println!("cargo:rerun-if-env-changed=PLATFORMIO_PACKAGES_DIR");
    println!("cargo:rustc-link-search={}", project_root.display());

    let common = |build: &mut cc::Build| {
        build
            .compiler(&gcc)
            .archiver(&ar)
            .include(project_root.join("c_support/tft/include"))
            .include(project_root.join("c_support/clock"))
            .include(&cmsis_core)
            .include(&cmsis_device)
            .flag("-mcpu=cortex-m4")
            .flag("-mthumb")
            .flag("-mfpu=fpv4-sp-d16")
            .flag("-mfloat-abi=hard")
            .flag("-ffunction-sections")
            .flag("-fdata-sections")
            .flag("-fno-builtin")
            .flag("-std=gnu11")
            .flag("-Wno-comment")
            .flag_if_supported("-Wno-old-style-declaration")
            .flag_if_supported("-Wno-maybe-uninitialized")
            .define("STM32F407xx", None)
            .define("HSE_VALUE", Some("8000000U"));
    };

    let mut board_support = cc::Build::new();
    common(&mut board_support);
    board_support.file(project_root.join("c_support/clock/hardware_system_init.c"));
    board_support.compile("board_support");

    let mut tft = cc::Build::new();
    common(&mut tft);
    for file in [
        "c_support/tft/src/stm324xg_lcd_ILI9341_Pro.c",
        "c_support/tft/src/stm32f4xx_fsmc.c",
        "c_support/tft/src/Fonts/font8.c",
        "c_support/tft/src/Fonts/font12.c",
        "c_support/tft/src/Fonts/font16.c",
        "c_support/tft/src/Fonts/font20.c",
        "c_support/tft/src/Fonts/font24.c",
        "c_support/tft/src/Fonts/fonts_16x24.c",
        "c_support/tft/src/Fonts/fonts_hz.c",
    ] {
        tft.file(project_root.join(file));
    }
    tft.compile("tft_support");
}

fn platformio_packages_dir() -> PathBuf {
    if let Ok(path) = env::var("PLATFORMIO_PACKAGES_DIR") {
        return PathBuf::from(path);
    }

    let home = env::var("HOME").expect("missing HOME environment variable");
    Path::new(&home).join(".platformio/packages")
}
