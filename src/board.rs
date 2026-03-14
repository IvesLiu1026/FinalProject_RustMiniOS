use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::asm::{nop, wfi};
use cortex_m::peripheral::{syst::SystClkSource, Peripherals};
use stm32f4::stm32f407::{self, gpioa};

const K1_PIN: u8 = 8;
const K0_PIN: u8 = 9;
const WKUP_PIN: u8 = 0;
const LED_PIN: u8 = 10;

const TOUCH_CS_PIN: u8 = 13;
const TOUCH_SCK_PIN: u8 = 0;
const TOUCH_MISO_PIN: u8 = 2;
const TOUCH_MOSI_PIN: u8 = 11;
const TOUCH_IRQ_PIN: u8 = 1;

const SYSTICK_HZ: u32 = 1000;
const CORE_CLOCK_HZ: u32 = 168_000_000;

type Gpio = gpioa::RegisterBlock;

static MILLIS: AtomicU32 = AtomicU32::new(0);

#[derive(Clone, Copy, Default)]
pub struct ButtonSnapshot {
    pub k1: bool,
    pub k0: bool,
    pub wkup: bool,
    pub k1_just_pressed: bool,
    pub k0_just_pressed: bool,
    pub wkup_just_pressed: bool,
}

impl ButtonSnapshot {
    pub fn home_chord(&self) -> bool {
        self.k0 && self.wkup
    }
}

pub struct Board {
    previous: ButtonSnapshot,
    led_on: bool,
}

impl Board {
    pub fn init() -> Self {
        enable_gpio_clocks();

        configure_input(gpiob(), K1_PIN, 0x1);
        configure_input(gpiob(), K0_PIN, 0x1);
        configure_input(gpioa(), WKUP_PIN, 0x2);
        configure_output(gpiof(), LED_PIN, 0x2);
        write_pin(gpiof(), LED_PIN, true);

        configure_output(gpioc(), TOUCH_CS_PIN, 0x3);
        configure_output(gpiob(), TOUCH_SCK_PIN, 0x3);
        configure_output(gpiof(), TOUCH_MOSI_PIN, 0x3);
        configure_input(gpiob(), TOUCH_MISO_PIN, 0x0);
        configure_input(gpiob(), TOUCH_IRQ_PIN, 0x1);

        write_pin(gpioc(), TOUCH_CS_PIN, true);
        write_pin(gpiob(), TOUCH_SCK_PIN, false);
        write_pin(gpiof(), TOUCH_MOSI_PIN, false);

        configure_systick();

        Self {
            previous: ButtonSnapshot::default(),
            led_on: false,
        }
    }

    pub fn poll_buttons(&mut self) -> ButtonSnapshot {
        let current = ButtonSnapshot {
            k1: read_active_low(gpiob(), K1_PIN),
            k0: read_active_low(gpiob(), K0_PIN),
            wkup: read_active_high(gpioa(), WKUP_PIN),
            k1_just_pressed: false,
            k0_just_pressed: false,
            wkup_just_pressed: false,
        };

        let snapshot = ButtonSnapshot {
            k1_just_pressed: current.k1 && !self.previous.k1,
            k0_just_pressed: current.k0 && !self.previous.k0,
            wkup_just_pressed: current.wkup && !self.previous.wkup,
            ..current
        };

        self.previous = current;
        snapshot
    }

    pub fn set_led(&mut self, on: bool) {
        self.led_on = on;
        write_pin(gpiof(), LED_PIN, !on);
    }

    pub fn toggle_led(&mut self) {
        self.set_led(!self.led_on);
    }

    pub fn led_on(&self) -> bool {
        self.led_on
    }
}

pub fn millis() -> u32 {
    MILLIS.load(Ordering::Relaxed)
}

pub fn delay_ms(duration_ms: u32) {
    let start = millis();
    while millis().wrapping_sub(start) < duration_ms {
        wfi();
    }
}

pub fn systick() {
    MILLIS.fetch_add(1, Ordering::Relaxed);
}

pub fn touch_select(selected: bool) {
    write_pin(gpioc(), TOUCH_CS_PIN, !selected);
    touch_half_cycle();
}

pub fn touch_transfer8(mut value: u8) -> u8 {
    let mut result = 0u8;

    for _ in 0..8 {
        write_pin(gpiof(), TOUCH_MOSI_PIN, (value & 0x80) != 0);
        touch_half_cycle();
        write_pin(gpiob(), TOUCH_SCK_PIN, true);
        touch_half_cycle();
        result =
            (result << 1) | u8::from((gpiob().idr().read().bits() & (1 << TOUCH_MISO_PIN)) != 0);
        write_pin(gpiob(), TOUCH_SCK_PIN, false);
        touch_half_cycle();
        value <<= 1;
    }

    result
}

pub fn touch_irq_active() -> bool {
    (gpiob().idr().read().bits() & (1 << TOUCH_IRQ_PIN)) == 0
}

fn configure_systick() {
    if let Some(mut cp) = Peripherals::take() {
        let reload = (CORE_CLOCK_HZ / SYSTICK_HZ) - 1;
        cp.SYST.set_clock_source(SystClkSource::Core);
        cp.SYST.set_reload(reload);
        cp.SYST.clear_current();
        cp.SYST.enable_interrupt();
        cp.SYST.enable_counter();
    }
}

fn enable_gpio_clocks() {
    let rcc = unsafe { &*stm32f407::RCC::ptr() };
    rcc.ahb1enr().modify(|_, w| {
        w.gpioaen()
            .set_bit()
            .gpioben()
            .set_bit()
            .gpiocen()
            .set_bit()
            .gpiofen()
            .set_bit()
    });
    let _ = rcc.ahb1enr().read().bits();
}

fn configure_input(port: &Gpio, pin: u8, pull: u32) {
    let shift = (pin as u32) * 2;
    port.moder()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0x3 << shift)) });
    port.otyper()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << pin)) });
    port.ospeedr()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0x3 << shift)) });
    port.pupdr()
        .modify(|r, w| unsafe { w.bits((r.bits() & !(0x3 << shift)) | (pull << shift)) });
}

fn configure_output(port: &Gpio, pin: u8, speed: u32) {
    let shift = (pin as u32) * 2;
    port.moder()
        .modify(|r, w| unsafe { w.bits((r.bits() & !(0x3 << shift)) | (0x1 << shift)) });
    port.otyper()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << pin)) });
    port.ospeedr()
        .modify(|r, w| unsafe { w.bits((r.bits() & !(0x3 << shift)) | (speed << shift)) });
    port.pupdr()
        .modify(|r, w| unsafe { w.bits(r.bits() & !(0x3 << shift)) });
}

fn read_active_low(port: &Gpio, pin: u8) -> bool {
    (port.idr().read().bits() & (1 << pin)) == 0
}

fn read_active_high(port: &Gpio, pin: u8) -> bool {
    (port.idr().read().bits() & (1 << pin)) != 0
}

fn write_pin(port: &Gpio, pin: u8, high: bool) {
    let bit = if high {
        1u32 << pin
    } else {
        1u32 << (pin as u32 + 16)
    };
    port.bsrr().write(|w| unsafe { w.bits(bit) });
}

fn touch_half_cycle() {
    nop();
    nop();
    nop();
    nop();
    nop();
    nop();
}

fn gpioa() -> &'static Gpio {
    unsafe { &*stm32f407::GPIOA::ptr().cast::<Gpio>() }
}

fn gpiob() -> &'static Gpio {
    unsafe { &*stm32f407::GPIOB::ptr().cast::<Gpio>() }
}

fn gpioc() -> &'static Gpio {
    unsafe { &*stm32f407::GPIOC::ptr().cast::<Gpio>() }
}

fn gpiof() -> &'static Gpio {
    unsafe { &*stm32f407::GPIOF::ptr().cast::<Gpio>() }
}
