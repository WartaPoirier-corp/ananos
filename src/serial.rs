use lazy_static::lazy_static;
use log::Level;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3f8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[macro_export]
macro_rules! dbg {
    ($($t:tt)*) => {
        {
            $crate::println!("{} = {:#b}", stringify!($($t)*), $($t)*);
            $($t)*
        }
    };
}

pub static LOGGER: SerialLogger = SerialLogger;

pub struct SerialLogger;

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // metadata.level() <= Level::Debug
        true
    }

    fn log(&self, record: &log::Record) {
        println!("logging");
        if self.enabled(record.metadata()) {
            println!("[{}] {}", record.metadata().level(), record.args());
        }
    }

    fn flush(&self) {}
}
