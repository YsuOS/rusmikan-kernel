use x86_64::{
    instructions::segmentation::*,
    registers::segmentation::SegmentSelector,
    structures::gdt::{Descriptor, GlobalDescriptorTable},
};

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
