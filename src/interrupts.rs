use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use crate::{println, print};
use core::ptr;
use crate::lapic::{init_lapic,disable_pic_8259,LAPIC,EOI};

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

// IRQ
const IRQ_OFFSET: u8 = 32;   // first 32 entries are reserved for exception by CPU
const IRQ_KBD: u32 = 1;

const REDTBL_MASKED: u32 = 0x00010000;

pub unsafe fn init() {
        init_idt();
        disable_pic_8259();
        init_lapic();
        init_io_apic();
        x86_64::instructions::interrupts::enable();
}

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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    _Timer = IRQ_OFFSET,
    Keyboard,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

unsafe fn init_idt(){
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT[InterruptIndex::Keyboard as usize]
        .set_handler_fn(keyboard_interrupt_handler);
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

    let mut kb = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(event)) = kb.add_byte(scancode) {
        if let Some(key) = kb.process_keyevent(event) {
            if let DecodedKey::Unicode(character) = key {
                print!("{}", character);
            }
        }
    }

    unsafe {
        *(EOI as *mut u32) = 0;
    }

}

