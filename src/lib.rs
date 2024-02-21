#![no_std]

use core::arch::asm;
use core::panic::PanicInfo;
use core::ptr;
use max78000_hal::debug::attach_debug;
use max78000_hal::gcr::{peripheral_reset, registers, system_clock_enable};
use max78000_hal::gpio::{GpioPin, OutputDriveStrength};
use max78000_hal::i2c::registers::Registers;
use max78000_hal::i2c::{NoPort, I2C};
use max78000_hal::memory_map::mmio;
use max78000_hal::uart::{BaudRates, CharacterLength, ParityValueSelect, StopBits, UART, UART0};
use max78000_hal::{debug_print, debug_println};

extern "C" {
    pub fn boot();
}

pub fn setup_uart() {
    // Set within the scope of this function.
    // DO NOT MESS WITH THIS STATIC
    static mut UART_DEBUG: Option<UART<UART0>> = None;

    // uart init
    let uart = UART::port_0_init(
        BaudRates::Baud115200,
        CharacterLength::EightBits,
        StopBits::OneBit,
        false,
        ParityValueSelect::OneBased,
        false,
    );

    // set static and attach debug
    unsafe { UART_DEBUG = Some(uart) };
    attach_debug(unsafe { UART_DEBUG.as_mut().unwrap() });
}

#[no_mangle]
pub extern "C" fn ap_function() {
    setup_uart();

    let mut trng = TRNG::init();
    loop {
        debug_println!("Random Number {}", trng.ready());

        if trng.ready() {
            panic!("trng data {}", trng.get_trng_data());
        }
    }

    unsafe { boot() };
}

#[no_mangle]
pub extern "C" fn comp_function() {
    unsafe { boot() }
}

fn delay() {
    unsafe {
        for _ in 0..1000000 {
            asm!("nop");
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let red = max78000_hal::gpio::hardware::led_red().unwrap();
    let green = max78000_hal::gpio::hardware::led_green().unwrap();
    let blue = max78000_hal::gpio::hardware::led_blue().unwrap();

    red.set_output(true);
    green.set_output(true);
    blue.set_output(true);
    loop {
        debug_println!("\n\n==========\nPANIC: {}", info);
        red.set_output(true);
        delay();
        red.set_output(false);
        delay();
    }
}
