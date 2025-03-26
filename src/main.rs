#![no_std] // dont link the Rust std
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)] // use custom test framework features
#![test_runner(ruost::test_utils::test_runner)] // set 'test_runner' function as test executor
#![reexport_test_harness_main = "test_main"] // set the name of the test framework entry function to test_main

extern crate alloc;

use core::panic::PanicInfo;
use ruost::{println, halt, init};
use bootloader::{BootInfo, entry_point};

// enables type checking of the entry point function
entry_point!(kernel_main);

// this function is the entry point - linker looks for a function named "_start" by default but entry_point macro handles that)
fn kernel_main(boot_info: &'static BootInfo) -> ! { // passed by bootloader, needed to use mapped physical memory
    init();
    println!("Hejka{}", "!");
    println!("Dziwko");

    use x86_64::VirtAddr;
    use ruost::memory;
    use ruost::allocator;
    use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

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