
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ruost::test_utils::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use ruost::allocator::HEAP_SIZE;
use core::panic::PanicInfo;
use alloc::boxed::Box;
use alloc::vec::Vec;

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    use ruost::allocator;

    ruost::init();
    allocator::init(boot_info);
    test_main();
    ruost::halt()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ruost::test_utils::test_panic_handler(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() { // would provoke out-of-memory failure if aloocator doesnt reuse memory
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}