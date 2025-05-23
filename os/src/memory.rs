use x86_64::{
    registers::control::Cr3, structures::paging::{FrameAllocator, OffsetPageTable, PageTable, PhysFrame, Size4KiB}, PhysAddr, VirtAddr
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // caller needs to guuarantee that passed memory map is valid
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0
        }
    }

    // returns iterator over usable frames specified in memory map
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        // get usable regions
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transfor to an iterator of frame start addresses
        let frame_addresses = addr_ranges // flat_map removes internal Itertator from step_by
            .flat_map(|r| r.step_by(4096)); // step through region address ranges by 4KiB
        // create PhysFrame types from start addresses
        frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

/// # Safety
/// this function is unsafe because the caller must guarantee that the complete physical memory is mapped to virtual memory at the passed physical_memory_offset
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    unsafe {
        let level_4_table = active_level_4_table(physical_memory_offset);
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

// returns a mutable reference to the active level 4 table.
// can only be called once
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr)
    -> &'static mut PageTable
{
    let (level_4_table_frame, _) = Cr3::read(); // Cr3 register points to level 4 page physical address

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe {&mut *page_table_ptr} // this unsafe is not mandatory, its here just to silence compiler warnings
}