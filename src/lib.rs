#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(test_utils::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![warn(
     clippy::all,
     clippy::pedantic,
     clippy::nursery,
 )]

pub mod gdt;
pub mod interrupts;
pub mod serial;
pub mod vga_buffer;
pub mod test_utils;

#[cfg(test)]
use core::panic::PanicInfo;

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable(); // executes 'sti' instruction (set interrupts)
}

pub fn halt() -> ! {
    loop { // energy-efficient endless loop
        x86_64::instructions::hlt();
    }
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    init();
    test_main();
    halt()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_utils::test_panic_handler(info)
}
