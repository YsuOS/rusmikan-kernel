use x86_64::structures::paging::page_table::{PageTable,PageTableFlags};
use x86_64::structures::paging::frame::PhysFrame;
use x86_64::structures::paging::PageSize;
use x86_64::structures::paging::page::{Size2MiB,Size1GiB};
use x86_64::registers::control::{Cr3,Cr3Flags};
use x86_64::addr::{PhysAddr,VirtAddr};

static mut PML4_TABLE: PageTable = PageTable::new();
static mut PDP_TABLE: PageTable = PageTable::new();

const EMPTY_PAGE_TABLE: PageTable = PageTable::new();
static mut PAGE_DIRECTORY: [PageTable; 64] = [EMPTY_PAGE_TABLE; 64];

pub unsafe fn init() {
    setup_identity_page_table();
    Cr3::write(get_phys_frame(&PML4_TABLE), Cr3Flags::empty());
}

fn get_phys_frame(page_table: &PageTable) -> PhysFrame {
    PhysFrame::from_start_address(
            PhysAddr::new(page_table as *const PageTable as u64)
        )
        .unwrap()
}

unsafe fn setup_identity_page_table() {
    // PML4: 1 entry
    // PDP : 512 entry
    // PD  : 64 entry
    // thus this page table supports 64 GB memory range
    // 1 * 512 * 64 * 2MB
    PML4_TABLE[0].set_frame(get_phys_frame(&PDP_TABLE), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

    for (i,d) in PAGE_DIRECTORY.iter_mut().enumerate() {
        PDP_TABLE[i].set_frame(get_phys_frame(d), PageTableFlags::PRESENT | PageTableFlags::WRITABLE);

        for (j,p) in PAGE_DIRECTORY[i].iter_mut().enumerate() {
            let addr = PhysAddr::new(i as u64 * Size1GiB::SIZE + j as u64 * Size2MiB::SIZE);
            p.set_addr(addr, PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE);
        }
    }
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame,_) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
