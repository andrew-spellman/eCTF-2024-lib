#![no_std]

use core::arch::asm;
use core::panic::PanicInfo;
use max78000_hal::debug::attach_debug;
use max78000_hal::debug_println;
use max78000_hal::trng::TRNG;
use max78000_hal::uart::{UART, UART0};

extern "C" {
    pub fn boot();
    pub fn LED_On(led_index: u32);
}

pub fn setup_uart() {
    // Set within the scope of this function.
    // DO NOT MESS WITH THIS STATIC
    static mut UART_DEBUG: Option<UART<UART0>> = None;

    // uart init
    let uart = UART::port_0_init();

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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe { LED_On(0) };
    loop {
        debug_println!("\n\n==========\nPANIC: {}", info);
        unsafe {
            for _ in 0..100000000 {
                asm!("nop");
            }
        }
    }
}
