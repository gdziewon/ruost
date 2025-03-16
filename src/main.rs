#![no_std] // dont link the Rust std
#![no_main] // disable all Rust-level entry points

mod vga_buffer;

use core::panic::PanicInfo;

// dont mangle the name of this function (compiler wont generate cryptic name, it will stay as "_start")
#[unsafe(no_mangle)] 
// this function is the entry point - linker looks for a function named "_start" by default
pub extern "C" fn _start() -> ! {
    println!("Hejka{}", "!");
    println!("Dziwko");
    
    loop {}
}

// function called on panic, we have to specify that bcs we dont have panic handling that comes with std
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}