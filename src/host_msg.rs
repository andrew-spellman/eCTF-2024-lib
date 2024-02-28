use core::ops::{Deref, DerefMut};

use max78000_hal::{
    debug::attach_debug,
    debug_println,
    error::{ErrorKind, Result},
    uart::{BaudRates, CharacterLength, ParityValueSelect, StopBits, UART, UART0},
};

struct HostMsg {
    uart: UART<UART0>,
    board_name: &'static str,
}

pub struct UartRef<'a>(&'a mut UART<UART0>);

impl<'a> Drop for UartRef<'a> {
    fn drop(&mut self) {
        unsafe { UART_REF = false };
    }
}

impl<'a> Deref for UartRef<'a> {
    type Target = UART<UART0>;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for UartRef<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl core::fmt::Write for HostMsg {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            match c {
                #[cfg(not(debug_assertions))]
                '\n' => self.uart.write_fmt(format_args!("\n\r"))?,
                #[cfg(debug_assertions)]
                '\n' => self
                    .uart
                    .write_fmt(format_args!("\n\r{}| ", self.board_name))?,
                c => self.uart.write_char(c)?,
            }
        }

        Ok(())
    }
}

static mut UART_DEBUG: Option<HostMsg> = None;

pub fn setup_uart(board_name: &'static str) {
    // uart init
    let uart = UART::port_0_init(
        BaudRates::Baud115200,
        CharacterLength::EightBits,
        StopBits::OneBit,
        false,
        ParityValueSelect::OneBased,
        false,
    )
    .unwrap();

    // set static and attach debug
    unsafe { UART_DEBUG = Some(HostMsg { uart, board_name }) };
    attach_debug(unsafe { UART_DEBUG.as_mut().unwrap() });
    debug_println!("\n");
}

static mut UART_REF: bool = false;

pub fn get_mut_uart() -> Option<UartRef<'static>> {
    if unsafe { UART_REF } {
        None
    } else {
        unsafe { UART_REF = true };

        let uart_ref = unsafe { &mut UART_DEBUG.as_mut()?.uart };
        Some(UartRef(uart_ref))
    }
}

#[macro_export]
macro_rules! host_msg {
    (Error, $($arg:tt)*) => {{
        max78000_hal::debug::_print(format_args!("%error: "));
        max78000_hal::debug::_print(format_args!($($arg)*));
        max78000_hal::debug::_print(format_args!("%\n"));
    }};
    (Success, $($arg:tt)*) => {{
        max78000_hal::debug::_print(format_args!("%success: "));
        max78000_hal::debug::_print(format_args!($($arg)*));
        max78000_hal::debug::_print(format_args!("%\n"));
    }};
    (Info, $($arg:tt)*) => {{
        max78000_hal::debug::_print(format_args!("%info: "));
        max78000_hal::debug::_print(format_args!($($arg)*));
        max78000_hal::debug::_print(format_args!("%\n"));
    }};
    (Debug, $($arg:tt)*) => {{
        max78000_hal::debug::_print(format_args!("%debug: "));
        max78000_hal::debug::_print(format_args!($($arg)*));
        max78000_hal::debug::_print(format_args!("%\n"));
    }};
    (Prompt, $($arg:tt)*) => {{
        max78000_hal::debug::_print(format_args!($($arg)*));
    }};
    (Ack) => {{
        max78000_hal::debug::_print(format_args!("%ack%\n"));
    }};
}

pub fn receive_msg(prompt: &str, rx_buffer: &mut [u8]) -> Result<usize> {
    host_msg!(Prompt, "{}", prompt);
    let mut rx_byte_count = 0;
    loop {
        match unsafe { UART_DEBUG.as_mut().unwrap().uart.read_receive_fifo() } {
            Ok(next_byte) => {
                if next_byte as char == '\r' || rx_byte_count == rx_buffer.len() {
                    break;
                }
                rx_buffer[rx_byte_count] = next_byte;
                rx_byte_count += 1;
            }
            Err(_) => (), // uart hasn't received any data
        }
    }
    if rx_byte_count == rx_buffer.len() {
        return Err(ErrorKind::Overflow);
    }
    Ok(rx_byte_count)
}

pub fn parse_args<'a>(rx_buffer: &'a [u8]) -> (Option<&'a str>, Option<&'a str>, Option<&'a str>) {
    let mut args = (None, None, None);
    let mut arg_count = 0;
    let result: core::result::Result<usize, usize> =
        rx_buffer
            .iter()
            .enumerate()
            .try_fold(0, |start_arg, (i, &byte)| match byte as char {
                ' ' if start_arg == i => Ok(start_arg + 1),
                ' ' => {
                    let arg =
                        Some(unsafe { core::str::from_utf8_unchecked(&rx_buffer[start_arg..i]) });
                    match arg_count {
                        0 => args.0 = arg,
                        1 => args.1 = arg,
                        2 => {
                            args.2 = arg;
                            return Err(start_arg);
                        }
                        _ => unreachable!("branch 2 would have returned err"),
                    }
                    arg_count += 1;
                    Ok(i + 1)
                }
                _ => Ok(start_arg),
            });
    let last_start_arg = match result {
        Ok(i) => i,
        Err(i) => i,
    };
    if last_start_arg != rx_buffer.len() {
        let next_space = rx_buffer[last_start_arg..]
            .iter()
            .take_while(|&&byte| byte as char != ' ')
            .count();
        let arg = Some(unsafe {
            core::str::from_utf8_unchecked(&rx_buffer[last_start_arg..last_start_arg + next_space])
        });
        match arg_count {
            0 => args.0 = arg,
            1 => args.1 = arg,
            2 => {
                args.2 = arg;
            }
            _ => (),
        }
    }
    args
}
