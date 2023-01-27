use acpi::platform::interrupt::Apic;

use crate::{
    acpi::get_bsp_info,
    interrupts::{IRQ_KBD, IRQ_OFFSET},
};
use core::ptr;

struct IoApic {
    ptr: *mut IoApicMmio,
}

impl IoApic {
    fn new(addr: u32) -> Self {
        Self {
            ptr: addr as *mut IoApicMmio,
        }
    }

    unsafe fn read(&self, reg: u32) -> u32 {
        ptr::write_volatile(&mut (*self.ptr).reg, reg);
        ptr::read_volatile(&mut (*self.ptr).data)
    }

    unsafe fn write(&self, reg: u32, data: u32) {
        ptr::write_volatile(&mut (*self.ptr).reg, reg);
        ptr::write_volatile(&mut (*self.ptr).data, data);
    }
}

// https://wiki.osdev.org/APIC#IO_APIC_Configuration
#[repr(C)]
struct IoApicMmio {
    reg: u32, // IOAPICBASE + 0x0
    pad: [u32; 3],
    data: u32, // IOAPICBASE + 0x10
}

// IOREGSEL Offset
const IOAPICID: u32 = 0x00000000;
const IOREDTBL: u32 = 0x00000010;

const REDTBL_MASKED: u32 = 0x00010000;

pub fn init_io_apic(apic: &Apic) {
    let bsp_lapic_id = get_bsp_info().local_apic_id;
    let ioapic = IoApic::new(apic.io_apics.first().unwrap().address);
    let max_intr = unsafe { ioapic.read(IOAPICID) } >> 16 & 0xFF;

    // Mark all interrupts edge-triggered, active high, disable, and not routed to any CPUs.
    for i in 0..max_intr {
        unsafe {
            ioapic.write(IOREDTBL + 2 * i, REDTBL_MASKED | (IRQ_OFFSET as u32 + i));
            ioapic.write(IOREDTBL + 2 * i + 1, 0);
        }
    }

    // Mark IRQ1 interrupt edge-triggered, active high, enable, and routed to the given cpunum
    unsafe {
        ioapic.write(IOREDTBL + 2 * IRQ_KBD, IRQ_OFFSET as u32 + IRQ_KBD);
        ioapic.write(IOREDTBL + 2 * IRQ_KBD + 1, bsp_lapic_id << 24);
    }
}
