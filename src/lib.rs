#![no_std]

mod ectf_params;
mod host_msg;

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

#[no_mangle]
pub extern "C" fn ap_function() {
    setup_uart("A");

    delay();
    debug_println!("Stuffs");
    delay();

    let mut uart2 = UART::port_2_init(
        BaudRates::Baud115200,
        CharacterLength::EightBits,
        StopBits::OneBit,
        false,
        ParityValueSelect::OneBased,
        false,
    )
    .unwrap();

    let uart0_ptr = mmio::UART_0;
    let uart2_ptr = mmio::UART_2;

    for offset in (0x00..=0x42).step_by(4) {
        let uart0_b = uart0_ptr + offset;
        let uart2_b = uart2_ptr + offset;
        let uart_val0 = unsafe { core::ptr::read_volatile(uart0_b as *const u32) };
        let uart_val2 = unsafe { core::ptr::read_volatile(uart2_b as *const u32) };
        let xor = uart_val0 ^ uart_val2;
        debug_println!(
            "UART -- 0x{:04x}: {:032b} {:032b} = {:032b}",
            offset,
            uart_val0,
            uart_val2,
            xor,
        );
    }

    loop {
        uart2.print_string("!\n");
        delay();
    }

    // let mut i2c = I2C::init_port_1_master().unwrap();
    // loop {
    //     debug_println!("I2C Master Transaction!");
    //     let mut bytes = [0u8; 4];
    //     let transmit_bytes = [0xBA, 0xDB, 0xAB, 0xEE];
    //     debug_println!(
    //         "{:#?}",
    //         i2c.master_transaction(0x23, Some(&mut bytes), Some(&transmit_bytes))
    //     );
    //     debug_println!("Got: {:#x?}", bytes);
    //     delay();
    // }
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
