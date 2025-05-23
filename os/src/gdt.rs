use x86_64::VirtAddr;
use x86_64::structures::{
    gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector},
    tss::TaskStateSegment,
};
use x86_64::instructions::{
    tables::load_tss,
    segmentation::{CS, Segment},
};
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
const STACK_SIZE: usize = 4096 * 5;

// FUN FACT: GDT was used for segmentation on older architectures (modern x86 dont use segmentation though)
lazy_static! { // we need to use GDT to load TSS structure
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = { // TODO: need to implement some scalable stack creation
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // allocate static stack

            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            stack_start + STACK_SIZE // write top address bcs stacks on x86 grow downwards
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub fn init() {
    GDT.0.load(); // load actual GDT
    unsafe {
        CS::set_reg(GDT.1.code_selector); // reload 'cs' register
        load_tss(GDT.1.tss_selector); // load TSS
    }
}