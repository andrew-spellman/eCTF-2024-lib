use crate::{
    ectf_params::{get_device, DeviceKind},
    flash, host_msg,
};
use max78000_hal::{
    debug_println,
    i2c::{I2CPort1, I2C},
};

pub fn list_cmd(i2c: &mut I2C<I2CPort1>) {
    let entry = flash::read().unwrap();
    for component_id in entry
        .component_ids
        .iter()
        .take(entry.component_count as usize)
    {
        host_msg!(Info, "P>0x{:08x}", component_id);
    }
    for i2c_address in 0x8..0x78 {
        // I2C Blacklist:
        // 0x18, 0x28, and 0x36 conflict with separate devices on MAX78000FTHR
        if i2c_address == 0x18 || i2c_address == 0x28 || i2c_address == 0x36 {
            continue;
        }
        match i2c.master_transaction(i2c_address, None, Some(&[0])) {
            Ok(()) => host_msg!(Info, "F>0x{:08x}", i2c_address),
            Err(_) => (),
        }
    }
    host_msg!(Success, "List");
}

pub fn boot_cmd() {
    let boot_msg = match get_device() {
        DeviceKind::ApplicationProcessor { boot_msg, .. } => boot_msg,
        _ => unreachable!("this function is only called by ap"),
    };
    host_msg!(Info, "AP>{}", boot_msg);
    host_msg!(Success, "Boot");
}

pub fn replace_cmd(rx_buffer: &[u8]) {
    let (token, component_in, component_out) = match host_msg::parse_args(rx_buffer) {
        (Some(token), Some(comp_in), Some(comp_out)) => {
            let comp_in = u32::from_str_radix(&comp_in[2..], 16).unwrap();
            let comp_out = u32::from_str_radix(&comp_out[2..], 16).unwrap();
            (token, comp_in, comp_out)
        }
        _ => {
            host_msg!(Error, "the input couldn't be parsed");
            return;
        }
    };
    debug_println!(
        "received: {}, {:x}, {:x}",
        token,
        component_in,
        component_out
    );
    host_msg!(Success, "Replace");
}

pub fn attest_cmd(rx_buffer: &[u8]) {
    let (pin, component) = match host_msg::parse_args(rx_buffer) {
        (Some(token), Some(comp), _) => {
            let comp = u32::from_str_radix(&comp[2..], 16).unwrap();
            (token, comp)
        }
        _ => {
            host_msg!(Error, "the input couldn't be parsed");
            return;
        }
    };
    debug_println!("received: {}, {:x}", pin, component);
    host_msg!(Success, "Attest");
}
