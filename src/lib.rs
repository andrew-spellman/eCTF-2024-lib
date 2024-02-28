#![no_std]

mod commands;
mod ectf_params;
mod flash;
mod host_msg;
mod security;

use crate::{
    commands::{attest_cmd, boot_cmd, list_cmd, replace_cmd},
    host_msg::{receive_msg, setup_uart},
};
use core::{arch::asm, panic::PanicInfo};
use max78000_hal::{
    gpio::hardware::{led_blue, led_green, led_red},
    i2c::I2C,
    trng::TRNG,
};

extern "C" {
    pub fn boot();
}

/// Returns the currently provisioned IDs and the number of provisioned IDs for
/// the current AP. This function is untilized in POST_BOOT functionality.
pub extern "C" fn get_provisioned_ids(buffer: *mut u32) -> i32 {
    _ = buffer;
    0
}

/// Securely send data over I2C. This function is utilized in POST_BOOT functionality.
pub extern "C" fn secure_send(i2c_address: u8, buffer: *const u8, len: u8) -> i32 {
    _ = (i2c_address, buffer, len);
    0
}

/// Securely receive data over I2C. This function is utilized in POST_BOOT functionality.
pub extern "C" fn secure_receive(i2c_address: u8, buffer: *mut u8) -> i32 {
    _ = (i2c_address, buffer);
    0
}

#[no_mangle]
pub extern "C" fn ap_function() {
    flash::init(0x4B1D).unwrap();
    setup_uart("A");

    let mut i2c = I2C::init_port_1_master().unwrap();

    let green = led_green().unwrap();
    green.set_output(false);

    let _random = TRNG::init().get_trng_data();

    host_msg!(Debug, "Application Processor Started");

    let mut uart_rx_buffer = [0u8; 100];
    loop {
        let uart_bytes_read = match receive_msg("Enter Command: ", &mut uart_rx_buffer) {
            Ok(bytes_read) => bytes_read,
            Err(_) => {
                host_msg!(Error, "UART Buffer Overflow");
                continue;
            }
        };

        // TODO: impl security::secure_master_transaction
        // TODO: impl our security tactic here using ^

        if &uart_rx_buffer[0..4] == "list".as_bytes() {
            list_cmd(&mut i2c);
        } else if &uart_rx_buffer[..4] == "boot".as_bytes() {
            boot_cmd();
        } else if &uart_rx_buffer[..7] == "replace".as_bytes() {
            replace_cmd(&uart_rx_buffer[..uart_bytes_read]);
        } else if &uart_rx_buffer[..6] == "attest".as_bytes() {
            attest_cmd(&uart_rx_buffer[..uart_bytes_read]);
        } else {
            host_msg!(Error, "Unrecognized command '{}'", unsafe {
                core::str::from_utf8_unchecked(&uart_rx_buffer[..uart_bytes_read])
            });
        }
    }
}

#[no_mangle]
pub extern "C" fn comp_function() {
    setup_uart("C");

    let i2c = I2C::init_port_1_slave(0x23).unwrap();
    _ = i2c;

    // TODO: impl security::secure_slave_transaction using a buffered
    // iterator adapter for the rx, and rx closures
    // TODO: impl our security tactic here using ^
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
    let red = led_red().unwrap();
    let green = led_green().unwrap();
    let blue = led_blue().unwrap();

    red.set_output(true);
    green.set_output(true);
    blue.set_output(true);
    loop {
        host_msg!(Error, "\n\n==========\nPANIC: {}", info);
        red.set_output(true);
        delay();
        red.set_output(false);
        delay();
    }
}
