use x86_64::instructions::port::Port;

// MMIO Address
pub const LAPIC: u32 = 0xFEE00000;
// LAPIC Register Address
const SVR: u32 = LAPIC + 0x000000F0;
pub const EOI: u32 = LAPIC + 0x000000B0;

const SVR_ENABLED: u32 = 0x00000100;

pub unsafe fn init_lapic() {
    let svr = SVR as *mut u32;
    *svr = SVR_ENABLED | 0xFF;
}

pub unsafe fn disable_pic_8259() {
    Port::new(0xa1).write(0xffu8);
    Port::new(0x21).write(0xffu8);
}
