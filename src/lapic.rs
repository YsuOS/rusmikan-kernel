use crate::{
    acpi::{get_apic_info, get_pm_timer_info, wait_milliseconds_with_pm_timer},
    interrupts::{IRQ_OFFSET, IRQ_TMR},
};
use core::ptr;
use spin::Lazy;
use x86_64::instructions::port::Port;

pub static LAPIC: Lazy<LApic> = Lazy::new(|| LApic::new(get_apic_info().local_apic_address as u32));

pub struct LApic {
    ptr: *mut u32,
}

unsafe impl Sync for LApic {}
unsafe impl Send for LApic {}

impl LApic {
    fn new(addr: u32) -> Self {
        Self {
            ptr: addr as *mut u32,
        }
    }

    unsafe fn read(&self, offset: u32) -> u32 {
        ptr::read_volatile((self.ptr as u32 + offset) as *mut u32)
    }

    pub unsafe fn write(&self, offset: u32, value: u32) {
        ptr::write_volatile((self.ptr as u32 + offset) as *mut u32, value);
    }

    unsafe fn start_lapic_timer(&self) {
        self.set_lapic_timer(u32::MAX);
    }

    unsafe fn stop_lapic_timer(&self) {
        self.set_lapic_timer(0);
    }

    unsafe fn set_lapic_timer(&self, time: u32) {
        self.write(TMRINITCNT, time);
    }

    unsafe fn lapic_timer_elapsed(&self) -> u32 {
        u32::MAX - self.read(TMRCURRCNT)
    }
}

// LAPIC Register Address
const SVR: u32 = 0x000000F0;
pub const EOI: u32 = 0x000000B0;
const LVT_TMR: u32 = 0x00000320;
const TMRINITCNT: u32 = 0x00000380;
const TMRCURRCNT: u32 = 0x00000390;
const TMRDIV: u32 = 0x000003e0;

const SVR_ENABLED: u32 = 0x00000100;
const X1: u32 = 0b1011; // divided by 1 (Divide Configuration Register)
const LVT_MASKED: u32 = 0x00010000;
const LVT_ONESHOT: u32 = 0x00000000;
const LVT_PERIODIC: u32 = 0x00020000;

static mut LAPIC_TMR_FREQ: u32 = 0;

pub fn init_lapic() {
    unsafe { LAPIC.write(SVR, SVR_ENABLED | 0xFF) };

    unsafe { init_lapic_timer() };
}

pub unsafe fn disable_pic_8259() {
    Port::new(0xa1).write(0xffu8);
    Port::new(0x21).write(0xffu8);
}

unsafe fn init_lapic_timer() {
    let pm_timer = get_pm_timer_info();
    LAPIC.write(TMRDIV, X1);
    LAPIC.write(LVT_TMR, LVT_ONESHOT | LVT_MASKED);

    LAPIC.start_lapic_timer();
    wait_milliseconds_with_pm_timer(pm_timer, 100);
    let elapsed = LAPIC.lapic_timer_elapsed();
    LAPIC.stop_lapic_timer();
    LAPIC_TMR_FREQ = elapsed * 10;

    LAPIC.write(LVT_TMR, LVT_PERIODIC | (IRQ_OFFSET as u32 + IRQ_TMR));
    LAPIC.write(TMRINITCNT, LAPIC_TMR_FREQ / 100);
}
