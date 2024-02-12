#![no_std]

use core::panic::PanicInfo;

extern "C" {
    pub fn boot();
}

#[no_mangle]
pub extern "C" fn rust_function() {
    unsafe { boot() }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
