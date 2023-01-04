use core::ptr;
use crate::{lapic::LAPIC, interrupts::{IRQ_KBD, IRQ_OFFSET}};

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
        ptr::read_volatile(&mut(*self.ptr).data)
    }

    unsafe fn write(&self, reg: u32, data: u32) {
        ptr::write_volatile(&mut (*self.ptr).reg, reg);
        ptr::write_volatile(&mut(*self.ptr).data, data);
    }
}

// https://wiki.osdev.org/APIC#IO_APIC_Configuration
#[repr(C)]
struct IoApicMmio {
    reg: u32,   // IOAPICBASE + 0x0
    pad: [u32; 3],
    data: u32,   // IOAPICBASE + 0x10
}

// MMIO Address
const IOAPIC: u32 = 0xFEC00000;
// IOREGSEL Offset
const IOAPICID: u32 = 0x00000000;
const IOREDTBL: u32 = 0x00000010;

const REDTBL_MASKED: u32 = 0x00010000;

pub unsafe fn init_io_apic() {
    let ioapic = IoApic::new(IOAPIC);
    let max_intr = ioapic.read(IOAPICID) >> 16 & 0xFF;

    // Mark all interrupts edge-triggered, active high, disable, and not routed to any CPUs.
    for i in 0..max_intr {
        ioapic.write(IOREDTBL+2*i, REDTBL_MASKED | (IRQ_OFFSET as u32 + i));
        ioapic.write(IOREDTBL+2*i+1, 0);
    }
    
    // FIXME: no supported SMP
    // current implementation get lapic_id from processor that the code is currently executing
    // on (BSP)
    let lapic_id = *(LAPIC as *mut u32) >> 24; // Get Local APIC ID

    // Mark IRQ1 interrupt edge-triggered, active high, enable, and routed to the given cpunum
    ioapic.write(IOREDTBL+2*IRQ_KBD, IRQ_OFFSET as u32 + IRQ_KBD);
    ioapic.write(IOREDTBL+2*IRQ_KBD+1, lapic_id << 24);
}
