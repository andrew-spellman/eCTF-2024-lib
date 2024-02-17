#![no_std]

use core::panic::PanicInfo;
use max78000_hal::debug::attach_debug;
use max78000_hal::debug_println;
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

    loop {
        debug_println!(
            "This is a test, of testing the test, for which I test the testing of test {}",
            unsafe { max78000_hal::SYSTEM_CORE_CLOCK }
        );
    }

    unsafe { boot() };
}

#[no_mangle]
pub extern "C" fn comp_function() {
    unsafe { boot() }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe { LED_On(0) };
    loop {}
}
