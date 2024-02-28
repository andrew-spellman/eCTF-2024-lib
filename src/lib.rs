#![no_std]

mod commands;
mod ectf_params;
mod flash;
mod host_msg;
mod security;

use crate::{
    commands::{attest_cmd, boot_cmd, list_cmd, replace_cmd},
    host_msg::{read_arg, setup_uart},
};
use core::{arch::asm, panic::PanicInfo, ptr::copy_nonoverlapping};
use max78000_hal::{
    aes::AES,
    gpio::hardware::{led_green, led_red},
    i2c::I2C,
    trng::TRNG,
};

#[no_mangle]
pub extern "C" fn ap_function() {
    flash::init(0x4B1D).unwrap();
    setup_uart("A");

    let mut i2c = I2C::init_port_1_master().unwrap();
    let mut aes = AES::init();
    let mut trng = TRNG::init();

    _ = led_green().unwrap().set_output(false);

    host_msg!(Debug, "Application Processor Started");

    loop {
        host_msg!(Debug, "Enter Command: ");
        let mut cmd_rx_buffer = [0; 7];
        let cmd_bytes_read = read_arg(&mut cmd_rx_buffer);
        if &cmd_rx_buffer[0..4] == "list".as_bytes() {
            list_cmd(&mut i2c);
            continue;
        }

        if &cmd_rx_buffer[0..4] == b"boot" {
            boot_cmd(i2c, aes, trng);
        }

        if &cmd_rx_buffer[0..7] == b"replace" {
            replace_cmd();
        } else if &cmd_rx_buffer[0..6] == b"attest" {
            attest_cmd(&mut i2c, &mut aes, &mut trng);
        } else {
            host_msg!(Error, "Unrecognized command '{}'", unsafe {
                core::str::from_utf8_unchecked(&cmd_rx_buffer[..cmd_bytes_read])
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

/// Returns the currently provisioned IDs and the number of provisioned IDs for
/// the current AP. This function is  in uninitialized functionality.
pub extern "C" fn get_provisioned_ids(buffer: *mut u32) -> i32 {
    let ids = flash::get_component_ids().unwrap();
    unsafe { copy_nonoverlapping(ids.as_ptr(), buffer, ids.len()) };
    ids.len() as i32
}

/// Securely send data over `I2C`. This function is utilized in `POST_BOOT` functionality.
pub extern "C" fn secure_send(i2c_address: u8, buffer: *const u8, len: u8) -> i32 {
    _ = (i2c_address, buffer, len);
    0
}

/// Securely receive data over `I2C`. This function is utilized in `POST_BOOT` functionality.
pub extern "C" fn secure_receive(i2c_address: u8, buffer: *mut u8) -> i32 {
    _ = (i2c_address, buffer);
    0
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

    red.set_output(true);
    loop {
        host_msg!(Error, "\n\n==========\nPANIC: {}", info);
        red.set_output(true);
        delay();
        red.set_output(false);
        delay();
    }
}
