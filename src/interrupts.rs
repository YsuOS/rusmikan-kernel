use crate::{
    acpi::get_apic_info,
    ioapic::init_io_apic,
    lapic::{disable_pic_8259, init_lapic, EOI, LAPIC},
    print, println, segment, serial_println, JIFFIES,
};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use x86_64::{
    instructions::port::Port,
    structures::idt::{InterruptDescriptorTable, InterruptStackFrame},
};

// IRQ
pub const IRQ_OFFSET: u8 = 32; // first 32 entries are reserved for exception by CPU
pub const IRQ_TMR: u32 = 0;
pub const IRQ_KBD: u32 = 1;

pub fn init() {
    init_idt();
    unsafe { disable_pic_8259() };
    let apic = get_apic_info();
    init_lapic(apic);
    init_io_apic(apic);
    x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = IRQ_OFFSET,
    Keyboard,
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(segment::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer as usize].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard as usize].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
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

    let lapic = LAPIC.get().unwrap();
    unsafe {
        lapic.write(EOI, 0);
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        JIFFIES += 1; // 1 tick
        println!("Timer Interrupt: {} tick", JIFFIES);
        let lapic = LAPIC.get().unwrap();
        lapic.write(EOI, 0);
    }
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}
