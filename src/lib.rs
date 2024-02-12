#![no_std]

use core::panic::PanicInfo;
use max78000_hal::i2c::I2C;

extern "C" {
    pub fn boot();
}

extern "C" {
    pub fn LED_On(led_index: u32);
}

#[no_mangle]
pub extern "C" fn ap_function() {
    let i2c_connection = I2C::port_0_init_master().unwrap();
    let tx = [0xDE, 0xAD, 0xBE, 0xEF];
    i2c_connection.master_transaction(0x10, None, Some(&tx)).unwrap();
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
