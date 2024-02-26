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
use max78000_hal::{debug_print, debug_println, trng};

extern "C" {
    pub fn boot();
}

pub fn setup_uart(str: &'static str) {
    // Set within the scope of this function.
    // DO NOT MESS WITH THIS STATIC
    static mut UART_DEBUG: Option<BetterDebug> = None;

    // uart init
    let mut uart = UART::port_0_init(
        BaudRates::Baud115200,
        CharacterLength::EightBits,
        StopBits::OneBit,
        false,
        ParityValueSelect::OneBased,
        false,
    )
    .unwrap();

    delay();
    uart.print_string("Connected...\n");

    struct BetterDebug {
        uart: UART<UART0>,
        str: &'static str,
    }

    impl core::fmt::Write for BetterDebug {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            for c in s.chars() {
                #[cfg(debug_assertions)]
                match c {
                    '\n' => self.uart.write_fmt(format_args!("\n{}| ", self.str))?,
                    c => self.uart.write_char(c)?,
                }
            }

            Ok(())
        }
    }

    // set static and attach debug
    unsafe { UART_DEBUG = Some(BetterDebug { uart, str }) };
    attach_debug(unsafe { UART_DEBUG.as_mut().unwrap() });
}

#[no_mangle]
pub extern "C" fn ap_function() {
    setup_uart("A");

    let mut i2c = I2C::init_port_1_master().unwrap();
    loop {
        debug_println!("I2C Master Transaction!");
        let mut bytes = [0u8; 4];
        let transmit_bytes = [0xBA, 0xDB, 0xAB, 0xEE];
        debug_println!(
            "{:#?}",
            i2c.master_transaction(0x23, Some(&mut bytes), Some(&transmit_bytes))
        );
        debug_println!("Got: {:#x?}", bytes);
        delay();
    }
}

#[no_mangle]
pub extern "C" fn comp_function() {
    setup_uart("C");

    let mut i2c = I2C::init_port_1_slave(0x23).unwrap();

    let dead_beef = [0xDE, 0xAD, 0xBE, 0xEF];

    loop {
        let mut iter = dead_beef.iter().copied();
        debug_println!(
            "{:#?}",
            i2c.slave_transaction(
                |byte| {
                    debug_println!("Got Byte: {}", byte);
                    Ok(())
                },
                || {
                    let byte = match iter.next() {
                        Some(byte) => byte,
                        None => {
                            iter = dead_beef.iter().copied();
                            iter.next().unwrap()
                        }
                    };
                    debug_println!("Sending Byte 0x{:02x}", byte);
                    Ok(byte)
                }
            )
        );
    }
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
