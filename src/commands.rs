use crate::{
    ectf_params::{get_device, DeviceKind},
    flash, host_msg,
};
use max78000_hal::i2c::{I2CPort1, I2C};

pub fn list_cmd(i2c: &mut I2C<I2CPort1>) {
    for component_id in match flash::get_component_ids() {
        Ok(ids) => ids,
        Err(e) => {
            host_msg!(Error, "Flash {:?}", e);
            return;
        }
    } {
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
    // TODO: move all held refs to statics for our c handlers to use
    unsafe {
        boot();
    }
}

extern "C" {
    fn boot();
}

pub fn replace_cmd(rx_buffer: &[u8]) {
    let (token, id_new, id_old) = {
        let mut split = core::str::from_utf8(rx_buffer).unwrap().split(" ");
        (
            split.next().unwrap(),
            u32::from_str_radix(&split.next().unwrap()[2..], 16).unwrap(),
            u32::from_str_radix(&split.next().unwrap()[2..], 16).unwrap(),
        )
    };
    host_msg!(Debug, "Received {}, {:x}, {:x}", token, id_new, id_old);
    match flash::swap_component(id_old, id_new) {
        Ok(()) => host_msg!(Success, "Replace"),
        Err(e) => host_msg!(Error, "Flash {:?}", e),
    }
}

pub fn attest_cmd(rx_buffer: &[u8]) {
    let (pin, component) = {
        let mut split = core::str::from_utf8(rx_buffer).unwrap().split(" ");
        (
            split.next().unwrap(),
            u32::from_str_radix(&split.next().unwrap()[2..], 16).unwrap(),
        )
    };
    host_msg!(Debug, "Received {}, {:x}", pin, component);
    host_msg!(Success, "Attest");
}
