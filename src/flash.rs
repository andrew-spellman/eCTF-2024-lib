use max78000_hal::error::{ErrorKind, Result};

#[repr(C)]
#[derive(Debug)]
pub struct FlashEntry {
    flash_magic: u32,
    pub component_count: u32,
    pub component_ids: [u32; 32],
}

extern "C" {
    fn init_flash(magic: u32) -> i32;
    fn read_flash() -> FlashEntry;
    fn write_flash(entry: &FlashEntry);
    fn poll_flash() -> i32;
}

static mut INITIALIZED: bool = false;

pub fn init(magic: u32) -> Result<()> {
    let result = unsafe { init_flash(magic) };
    match result {
        0 => {
            unsafe { INITIALIZED = true };
            Ok(())
        }
        1 => Err(ErrorKind::Fail),
        _ => unreachable!(),
    }
}

pub fn read() -> Result<FlashEntry> {
    if !unsafe { INITIALIZED } {
        return Err(ErrorKind::Uninitialized);
    }
    unsafe {
        let entry = read_flash();
        match poll_flash() {
            0 => Ok(entry),
            1 => Err(ErrorKind::Fail),
            _ => unreachable!(),
        }
    }
}

pub fn write(entry: &FlashEntry) -> Result<()> {
    if !unsafe { INITIALIZED } {
        return Err(ErrorKind::Uninitialized);
    }
    unsafe {
        write_flash(entry);
        match poll_flash() {
            0 => Ok(()),
            1 => Err(ErrorKind::Fail),
            _ => unreachable!(),
        }
    }
}
