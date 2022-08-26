use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor};
use x86_64::instructions::segmentation::*;
use x86_64::registers::segmentation::SegmentSelector;

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();

pub unsafe fn init() {
    let code_selector = GDT.add_entry(Descriptor::kernel_code_segment());
    let data_selector = GDT.add_entry(Descriptor::kernel_data_segment());
    GDT.load();
    CS::set_reg(code_selector);
    SS::set_reg(data_selector);
    DS::set_reg(SegmentSelector::NULL);
    ES::set_reg(SegmentSelector::NULL);
    FS::set_reg(SegmentSelector::NULL);
    GS::set_reg(SegmentSelector::NULL);
}
