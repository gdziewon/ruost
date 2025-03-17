use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::fmt::Write;

const SERIAL1_PORT: u16 = 0x3F8;

// macros similar to VGA buffer ones
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    SERIAL1.lock().write_fmt(args).expect("Printing to serial failed");
}

// like in VGA text buffer, lazy_static and spinlock is used
lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(SERIAL1_PORT) }; // 0x3F8 is a standard port number for first serial iface
        serial_port.init(); // lazy_static ensures this init is called only once
        Mutex::new(serial_port)
    };
}