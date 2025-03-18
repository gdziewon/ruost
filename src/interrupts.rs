use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;

use crate::{println, print};
use crate::gdt::DOUBLE_FAULT_IST_INDEX;

pub const PIC_1_OFFSET: u8 = 32; // 32 so it wont overlap with exception handler values
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

// interrupt controllers piar
pub static PICS: Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

lazy_static! { // IDT is a table with addresses to functions that CPU should execute when it encounters an exception
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new(); // each exception has it's own entry
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX); // needed in case of stack overflow, handler should use fresh stack
        }
        
         // CPU reacts identically to exceptions and external interrupts
        idt[InterruptIndex::Timer.as_usize()] // InterruptDescriptorTable implements IndexMut so array indexing syntax works
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)] // C-like enum
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET, // programmable interval timer
    Keyboard, // defaults to previous value +1

}

impl InterruptIndex {
    const fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame)
{    
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// the most important exception handler - kind of catch all - prevents triple fault
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> !
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    //print!(".");
    
    end_of_interrupt(InterruptIndex::Timer);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(
    _stack_frame: InterruptStackFrame)
{
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(ScancodeSet1::new(),
                layouts::Us104Key, HandleControl::Ignore)
            );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); // data port of the keyboard controller (PS/2)
    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            match key {
                DecodedKey::Unicode(character) => print!("{}", character),
                DecodedKey::RawKey(key) => print!("{:?}", key),
            }
        }
    }

    end_of_interrupt(InterruptIndex::Keyboard);
}

fn end_of_interrupt(interrupt_id: InterruptIndex) {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(interrupt_id.as_u8());
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
