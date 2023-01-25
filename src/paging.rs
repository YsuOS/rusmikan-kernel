use spin::Mutex;
use x86_64::{
    addr::{PhysAddr, VirtAddr},
    registers::control::{Cr3, Cr3Flags},
    structures::paging::{
        frame::PhysFrame,
        page::{Size1GiB, Size2MiB},
        page_table::{PageTable, PageTableFlags},
        PageSize,
    },
};

static PML4_TABLE: Mutex<PageTable> = Mutex::new(PageTable::new());
static PDP_TABLE: Mutex<PageTable> = Mutex::new(PageTable::new());

const EMPTY_PAGE_TABLE: PageTable = PageTable::new();
const PD_TABLE_CNT: usize = 64;
static PD_TABLE: Mutex<[PageTable; PD_TABLE_CNT]> = Mutex::new([EMPTY_PAGE_TABLE; PD_TABLE_CNT]);

pub fn init() {
    setup_identity_page_table();
    unsafe {
        Cr3::write(get_phys_frame(&PML4_TABLE.lock()), Cr3Flags::empty());
    }
}

fn get_phys_frame(page_table: &PageTable) -> PhysFrame {
    PhysFrame::from_start_address(PhysAddr::new(page_table as *const PageTable as u64)).unwrap()
}

fn setup_identity_page_table() {
    // PML4: 1 entry/512 entry = 1 PDP Table
    // PDP : 64 entry/512 entry = 64 PD Table
    // PD  : 512 entry/512 entry = 512 * 2MB Address
    // thus this page table supports 64 GB memory range
    // 1 * 64 * 512 * 2MB
    let mut pml4_table = PML4_TABLE.lock();
    pml4_table[0].set_frame(
        get_phys_frame(&PDP_TABLE.lock()),
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
    );

    for (i, pd_table) in PD_TABLE.lock().iter_mut().enumerate() {
        let mut pdp_table = PDP_TABLE.lock();
        pdp_table[i].set_frame(
            get_phys_frame(pd_table),
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
        );
    }

    for i in 0..PD_TABLE_CNT {
        let mut pd_table = PD_TABLE.lock();
        for (j, entry) in pd_table[i].iter_mut().enumerate() {
            let addr = PhysAddr::new(i as u64 * Size1GiB::SIZE + j as u64 * Size2MiB::SIZE);
            entry.set_addr(
                addr,
                PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::HUGE_PAGE,
            );
        }
    }
}

pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}
