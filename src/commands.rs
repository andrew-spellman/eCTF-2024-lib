use core::str::from_utf8_unchecked;

use crate::{
    ectf_params::{get_device, DeviceKind},
    flash, host_msg,
    host_msg::read_arg,
};
use max78000_hal::{
    aes::AES,
    error::ErrorKind,
    i2c::{I2CPort1, I2C},
    trng::TRNG,
};

enum _ComponentCommand {
    Boot,
    Attest,
}

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

pub fn boot_cmd(i2c: I2C<I2CPort1>, aes: AES, trng: TRNG) -> ! {
    let boot_msg = match get_device() {
        DeviceKind::ApplicationProcessor { boot_msg, .. } => boot_msg,
        _ => unreachable!("boot_cmd() is only called by ap"),
    };

    host_msg!(Info, "AP>{}", boot_msg);
    host_msg!(Success, "Boot");

    _ = (i2c, aes, trng);

    // TODO: move all held refs to statics for our c handlers to use
    // TODO: securly  boot components

    unsafe { boot() }
}

extern "C" {
    fn boot() -> !;
}

pub fn replace_cmd() {
    host_msg!(Ack);

    let mut token_buffer = [0; 16];
    let mut id_new_buffer = [0; 16];
    let mut id_old_buffer = [0; 16];

    let (token, id_new, id_old) = {
        let token_len = read_arg(&mut token_buffer);
        host_msg!(Ack);
        let id_new_len = read_arg(&mut id_new_buffer);
        host_msg!(Ack);
        let id_old_len = read_arg(&mut id_old_buffer);
        (
            unsafe { from_utf8_unchecked(&token_buffer[..token_len]) },
            u32::from_str_radix(
                &unsafe { from_utf8_unchecked(&id_new_buffer) }[2..id_new_len],
                16,
            )
            .unwrap(),
            u32::from_str_radix(
                &unsafe { from_utf8_unchecked(&id_old_buffer) }[2..id_old_len],
                16,
            )
            .unwrap(),
        )
    };

    if token
        != match get_device() {
            DeviceKind::ApplicationProcessor { ap_token, .. } => ap_token,
            _ => unreachable!("boot_cmd() is only called by ap"),
        }
    {
        host_msg!(Error, "Incorrect Token");
        return;
    }

    match flash::swap_component(id_old, id_new) {
        Ok(()) => host_msg!(Success, "Replace"),
        Err(ErrorKind::BadParam) => host_msg!(Error, "Component not found"),
        Err(e) => host_msg!(Error, "Flash {:?}", e),
    }
}

pub fn attest_cmd(i2c: &mut I2C<I2CPort1>, aes: &mut AES, trng: &mut TRNG) {
    host_msg!(Ack);
    let mut pin_buffer = [0; 6];
    let mut component_buffer = [0; 16];
    let (pin, component) = {
        let pin_len = read_arg(&mut pin_buffer);
        host_msg!(Ack);
        let component_len = read_arg(&mut component_buffer);
        (
            unsafe { from_utf8_unchecked(&pin_buffer[..pin_len]) },
            u32::from_str_radix(
                &unsafe { from_utf8_unchecked(&component_buffer) }[2..component_len],
                16,
            )
            .unwrap(),
        )
    };

    if pin
        != match get_device() {
            DeviceKind::ApplicationProcessor { ap_pin, .. } => ap_pin,
            _ => unreachable!("boot_cmd() is only called by ap"),
        }
    {
        host_msg!(Error, "Incorrect Pin");
        return;
    }

    // TODO: securly get data from components
    _ = (i2c, aes, trng);

    let attestation_loc = "";
    let attestation_date = "";
    let attestation_customer = "";

    host_msg!(Info, "C>0x{:x}", component);
    host_msg!(Info, "LOC>{}", attestation_loc);
    host_msg!(Info, "DATE>{}", attestation_date);
    host_msg!(Info, "CUST>{}", attestation_customer);
    host_msg!(Success, "Attest");
}
