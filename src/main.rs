#![no_std]
#![no_main]

mod app_registry;
mod apps;
mod assets;
mod board;
mod companion;
mod display;
mod dungeon;
mod font;
mod font_zh;
mod jpeg_demo;
mod media;
mod shell;
mod shell_contract;
mod storage;
mod storage_codec;
mod system_info;
mod touch;
mod ui;

use board::{millis, Board};
use cortex_m_rt::{entry, exception};
use display::Display;
use panic_halt as _;
use shell::{boot_sequence, MiniOs};
use stm32f4::stm32f407::interrupt;
use touch::Touch;

unsafe extern "C" {
    fn stm32f4_Hardware_Init();
}

#[entry]
fn main() -> ! {
    enable_fpu();
    unsafe {
        stm32f4_Hardware_Init();
    }

    let mut board = Board::init();
    let mut touch = Touch::new();
    let mut display = Display::init();
    let mut os = MiniOs::new();
    let startup_buttons = board.poll_buttons();
    let safe_boot_requested = startup_buttons.k1;

    if safe_boot_requested {
        os.enter_safe_mode();
    } else if let Some(state) = storage::load() {
        os.apply_persisted_state(state, &mut touch);
    }

    boot_sequence(
        &mut display,
        os.theme(),
        safe_boot_requested,
        os.touch_ready(),
    );
    board.set_led(false);
    let touch_state = touch.state();
    os.render(&mut display, &board, &touch_state, true);

    let mut last_frame = millis();

    loop {
        let now = millis();
        let dt = now.wrapping_sub(last_frame);
        if dt < 16 {
            cortex_m::asm::wfi();
            continue;
        }

        last_frame = now;
        let sim_dt = dt.min(25);
        os.update_frame_timing(dt);
        let buttons = board.poll_buttons();
        let touch_state = touch.update(sim_dt as u16);
        let dirty = os.update(&mut board, &buttons, &touch_state, &mut touch, sim_dt);
        os.service_background_tasks(&buttons, &touch_state);
        let full_refresh = os.take_full_redraw();
        if dirty || full_refresh {
            os.render(&mut display, &board, &touch_state, full_refresh);
        }
    }
}

#[exception]
fn SysTick() {
    board::systick();
}

#[interrupt]
fn EXTI1() {
    board::touch_irq_exti_handler();
}

#[interrupt]
fn EXTI0() {
    board::wkup_button_exti_handler();
}

#[interrupt]
fn EXTI9_5() {
    board::button_bank_exti_handler();
}

fn enable_fpu() {
    unsafe {
        let cpacr = 0xE000_ED88 as *mut u32;
        let current = core::ptr::read_volatile(cpacr);
        core::ptr::write_volatile(cpacr, current | (0b1111 << 20));
    }
}
