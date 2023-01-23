use crate::{
    acpi::wait_milliseconds_with_pm_timer,
    interrupts::{IRQ_OFFSET, IRQ_TMR},
};
use core::ptr;
use x86_64::instructions::port::Port;

// MMIO Address
pub const LAPIC: u32 = 0xFEE00000;
// LAPIC Register Address
const SVR: u32 = LAPIC + 0x000000F0;
pub const EOI: u32 = LAPIC + 0x000000B0;
const LVT_TMR: u32 = LAPIC + 0x00000320;
const TMRINITCNT: u32 = LAPIC + 0x00000380;
const TMRCURRCNT: u32 = LAPIC + 0x00000390;
const TMRDIV: u32 = LAPIC + 0x000003e0;

const SVR_ENABLED: u32 = 0x00000100;
const X1: u32 = 0b1011; // divided by 1 (Divide Configuration Register)
const LVT_MASKED: u32 = 0x00010000;
const LVT_ONESHOT: u32 = 0x00000000;
const LVT_PERIODIC: u32 = 0x00020000;

static mut LAPIC_TMR_FREQ: u32 = 0;

pub unsafe fn init_lapic() {
    let svr = SVR as *mut u32;
    *svr = SVR_ENABLED | 0xFF;

    init_lapic_timer();
}

pub unsafe fn disable_pic_8259() {
    Port::new(0xa1).write(0xffu8);
    Port::new(0x21).write(0xffu8);
}

unsafe fn init_lapic_timer() {
    let lvt_timer = LVT_TMR as *mut u32;
    let timer_div = TMRDIV as *mut u32;
    let timer_init_cnt = TMRINITCNT as *mut u32;
    *timer_div = X1;
    *lvt_timer = LVT_ONESHOT | LVT_MASKED;

    start_lapic_timer();
    wait_milliseconds_with_pm_timer(100);
    let elapsed = lapic_timer_elapsed();
    stop_lapic_timer();
    LAPIC_TMR_FREQ = elapsed * 10;

    *lvt_timer = LVT_PERIODIC | (IRQ_OFFSET as u32 + IRQ_TMR);
    ptr::write_volatile(timer_init_cnt, LAPIC_TMR_FREQ / 100);
}

unsafe fn start_lapic_timer() {
    let timer_init_cnt = TMRINITCNT as *mut u32;
    ptr::write_volatile(timer_init_cnt, u32::MAX);
}

unsafe fn stop_lapic_timer() {
    let timer_init_cnt = TMRINITCNT as *mut u32;
    ptr::write_volatile(timer_init_cnt, 0);
}

unsafe fn lapic_timer_elapsed() -> u32 {
    let timer_current_cnt = TMRCURRCNT as *mut u32;
    u32::MAX - ptr::read_volatile(timer_current_cnt)
}
