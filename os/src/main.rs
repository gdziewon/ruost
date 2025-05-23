#![no_std] // dont link the Rust std
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // use custom test framework features
#![test_runner(ruost::test_utils::test_runner)] // set 'test_runner' function as test executor
#![reexport_test_harness_main = "test_main"] // set the name of the test framework entry function to test_main

extern crate alloc;

use core::panic::PanicInfo;
use ruost::{halt, init, println};
use bootloader::{BootInfo, entry_point};

// enables type checking of the entry point function
entry_point!(kernel_main);

// this function is the entry point - linker looks for a function named "_start" by default but entry_point macro handles that)
fn kernel_main(boot_info: &'static BootInfo) -> ! { // passed by bootloader, needed to use mapped physical memory
    init();
    println!("Hejka{}", "!");

    use ruost::allocator;
    use ruost::task::keyboard;
    use ruost::task::executor::Executor;
    use ruost::task::Task;
    
    allocator::init(boot_info);

    #[cfg(test)]
    test_main();
    
    println!("Przeszlo!");

    let mut executor = Executor::new(); // new
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypress()));
    executor.run();

    halt()
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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