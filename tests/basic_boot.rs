// all integration tests are their own executables and completely separate from main.rs
// this means that each test needs to define its own entry point function

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ruost::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use ruost::{println, halt};


#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main(); // no need for cfg(test) attributes because integration test executables are always built test mode

    halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ruost::test_panic_handler(info)
}

#[test_case]
fn test_println() {
    println!("test_println output");
}