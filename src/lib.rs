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
use max78000_hal::{debug_print, debug_println};

extern "C" {
    pub fn boot();
}

pub fn setup_uart() {
    // Set within the scope of this function.
    // DO NOT MESS WITH THIS STATIC
    static mut UART_DEBUG: Option<UART<UART0>> = None;

    // uart init
    let uart = UART::port_0_init(
        BaudRates::Baud115200,
        CharacterLength::EightBits,
        StopBits::OneBit,
        false,
        ParityValueSelect::OneBased,
        false,
    );

    // set static and attach debug
    unsafe { UART_DEBUG = Some(uart) };
    attach_debug(unsafe { UART_DEBUG.as_mut().unwrap() });
}

#[no_mangle]
pub extern "C" fn ap_function() {
    setup_uart();

    delay();
    debug_println!("Resetting GPIO1");
    peripheral_reset(max78000_hal::gcr::HardwareSource::GPIO1);
    debug_println!("Resetting I2C1");
    peripheral_reset(max78000_hal::gcr::HardwareSource::I2C1);

    system_clock_enable(max78000_hal::gcr::HardwareSource::GPIO0, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::GPIO1, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::I2C1, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::I2C0, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::I2C2, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::CPU1, true);
    system_clock_enable(max78000_hal::gcr::HardwareSource::DMA, true);
    delay();

    // let i2c_scl = GpioPin::new(max78000_hal::gpio::GpioSelect::Gpio0, 16).unwrap();
    // let i2c_sda = GpioPin::new(max78000_hal::gpio::GpioSelect::Gpio0, 17).unwrap();

    // i2c_scl.configure_output(
    //     OutputDriveStrength::Strength0(max78000_hal::gpio::VoltageSelect::VddIO),
    //     max78000_hal::gpio::PinFunction::AF1,
    // );
    // i2c_sda.configure_output(
    //     OutputDriveStrength::Strength0(max78000_hal::gpio::VoltageSelect::VddIO),
    //     max78000_hal::gpio::PinFunction::AF1,
    // );

    // i2c_scl.configure_input(
    //     max78000_hal::gpio::ResistorStrength::None,
    //     max78000_hal::gpio::PinFunction::AF1,
    // );
    // i2c_sda.configure_input(
    //     max78000_hal::gpio::ResistorStrength::None,
    //     max78000_hal::gpio::PinFunction::AF1,
    // );

    // let mut i2c_registers = Registers::new(mmio::I2C_PORT_1);

    // unsafe {
    //     i2c_registers.set_software_i2c_mode(true);
    //     i2c_registers.set_i2c_peripheral_enable(true);
    // }

    // let dingus_i2c_ctrl = unsafe { (mmio::I2C_PORT_1 as *mut u32) };

    // unsafe {
    // let fuck = (mmio::GLOBAL_CONTROL + 0x24) as *mut u32;
    // let fuck2 = (mmio::GLOBAL_CONTROL + 0x48) as *mut u32;
    //
    // ptr::write_volatile(fuck, 4294967263);
    // ptr::write_volatile(fuck2, 0);
    // }
    let mut i2c = I2C::init_port_1_master().unwrap();

    loop {
        debug_println!("I2C Master Transaction!");
        i2c.master_transaction(0x01, None, Some(&[0xDE, 0xED, 0xBE, 0xEF]));
        i2c.master_transaction(0x02, None, Some(&[0x00, 0x00, 0x00, 0x00]));
        i2c.master_transaction(0x03, None, Some(&[0x0F, 0xF0, 0x00, 0xFF]));
        // unsafe {
        //     ptr::write_volatile(dingus_i2c_ctrl, 1 << 10 | 1 | 1 << 31);
        //     debug_println!("Dingus I2C: {}", ptr::read_volatile(dingus_i2c_ctrl));
        // }
        // unsafe { i2c_registers.set_i2c_peripheral_enable(true) };
        // debug_println!("Dingus: {}", i2c_registers.get_fifo_data());
        // debug_println!(
        //     "Should be true: {}",
        //     i2c_registers.get_transmit_fifo_empty()
        // );
        // debug_println!("Enabled : {}", i2c_registers.get_i2c_peripheral_enable());
        // unsafe { i2c_registers.set_scl_hardware_pin_released(false) };
        // unsafe { i2c_registers.set_sda_hardware_pin_released(false) };
        // debug_println!(
        //     "True : {}, {} == {}, {}",
        //     i2c_registers.get_scl_pin(),
        //     i2c_registers.get_sda_pin(),
        //     i2c_registers.get_scl_hardware_pin_released(),
        //     i2c_registers.get_sda_hardware_pin_released()
        // );
        // delay();
        // unsafe { i2c_registers.set_scl_hardware_pin_released(true) };
        // unsafe { i2c_registers.set_sda_hardware_pin_released(true) };
        // debug_println!(
        //     "False : {}, {} == {}, {}",
        //     i2c_registers.get_scl_pin(),
        //     i2c_registers.get_sda_pin(),
        //     i2c_registers.get_scl_hardware_pin_released(),
        //     i2c_registers.get_sda_hardware_pin_released()
        // );
        // delay();
        debug_println!("CUM");
        delay();
    }

    unsafe { boot() };
}

#[no_mangle]
pub extern "C" fn comp_function() {
    unsafe { boot() }
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
