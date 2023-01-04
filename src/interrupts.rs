use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::port::Port;
use crate::ioapic::init_io_apic;
use crate::{println, print, JIFFIES};
use crate::lapic::{init_lapic,disable_pic_8259,EOI};

// IRQ
pub const IRQ_OFFSET: u8 = 32;   // first 32 entries are reserved for exception by CPU
pub const IRQ_TMR: u32 = 0;
pub const IRQ_KBD: u32 = 1;

pub unsafe fn init() {
        init_idt();
        disable_pic_8259();
        init_lapic();
        init_io_apic();
        x86_64::instructions::interrupts::enable();
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = IRQ_OFFSET,
    Keyboard,
}

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

unsafe fn init_idt(){
    IDT.breakpoint.set_handler_fn(breakpoint_handler);
    IDT[InterruptIndex::Timer as usize]
        .set_handler_fn(timer_interrupt_handler);
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

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        println!("Timer Interrupt: {} tick", JIFFIES);
        JIFFIES += 1;   // 1 tick
        *(EOI as *mut u32) = 0;
    }
}
