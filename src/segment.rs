use lazy_static::lazy_static;
use x86_64::{
    instructions::{segmentation::*, tables::load_tss},
    registers::segmentation::SegmentSelector,
    structures::{
        gdt::{Descriptor, GlobalDescriptorTable},
        tss::TaskStateSegment,
    },
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub unsafe fn init() {
    let code_selector = GDT.add_entry(Descriptor::kernel_code_segment());
    let data_selector = GDT.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));
    GDT.load();
    CS::set_reg(code_selector);
    SS::set_reg(data_selector);
    DS::set_reg(SegmentSelector::NULL);
    ES::set_reg(SegmentSelector::NULL);
    FS::set_reg(SegmentSelector::NULL);
    GS::set_reg(SegmentSelector::NULL);
    load_tss(tss_selector);
}
