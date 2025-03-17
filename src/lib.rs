#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

const TEST_IOBASE_PORT: u16 = 0xf4;

pub mod serial;
pub mod vga_buffer;

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(), // implement on functions
{
    fn run(&self) {
        // any::typename is function implemented in compiler, returns string description of every type
        serial_print!("{}...\t", core::any::type_name::<T>()); // for functions, type is their name
        self(); // invoke function, this works because we require self to implement Fn() trait
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) { // &[&dyn Testable()] is a slice of trait object references of the Testable() trait
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[fail]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failure);
    halt()
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)] // represent variants as u32 integers
pub enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode){
    use x86_64::instructions::port::Port;

    unsafe { // unsafe bcs writing to an I/O port can generally result in arbitrary behavior
        let mut port = Port::new(TEST_IOBASE_PORT); // 0xf4 is the value of iobase arg
        port.write(exit_code as u32); // u32 bcs iosize byte equals 4bytes
    }
}

pub fn halt() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}