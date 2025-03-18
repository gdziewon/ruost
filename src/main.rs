#![no_std] // dont link the Rust std
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // use custom test framework features
#![test_runner(ruost::test_utils::test_runner)] // set 'test_runner' function as test executor
#![reexport_test_harness_main = "test_main"] // set the name of the test framework entry function to test_main

use core::panic::PanicInfo;
use ruost::{println, halt, init};

// dont mangle the name of this function (compiler wont generate cryptic name, it will stay as "_start")
#[unsafe(no_mangle)] 
// this function is the entry point - linker looks for a function named "_start" by default
pub extern "C" fn _start() -> ! {
    init();
    println!("Hejka{}", "!");
    println!("Dziwko");

    #[cfg(test)]
    test_main();
    
    println!("Nie wyjebalo!");
    halt()
}

// function called on panic, we have to specify that bcs we dont have panic handling that comes with std
#[cfg(not(test))] // don't include in testing
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt()
}

#[cfg(test)]
#[panic_handler] // panic handler for test mode
fn panic(info: &PanicInfo) -> ! {
    ruost::test_utils::test_panic_handler(info)
}