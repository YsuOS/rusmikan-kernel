#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod graphics;
mod ascii_font;
mod console;
mod pci;
mod serial;
mod interrupts;
mod segment;
mod paging;
mod mm;
mod lapic;
mod ioapic;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::{FrameBufferConfig,MemoryMap};
use graphics::{Graphic, Rgb};
use core::fmt::Write;
use console::CONSOLE;
use pci::list_pci_devices;
use mm::{BitMapMemoryManager,BITMAP_MEMORY_MANAGER};

use crate::lapic::{start_lapic_timer, stop_lapic_timer, lapic_timer_elapsed};

const BG_COLOR: Rgb = Rgb { r: 241, g:141, b:0 };

pub static mut JIFFIES: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("{}", info);
    exit_qemu(QemuExitCode::Failed);
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[repr(align(16))]
struct KernelMainStack([u8; 1024 * 1024]);

#[no_mangle]
static mut KERNEL_MAIN_STACK: KernelMainStack = KernelMainStack([0; 1024 * 1024]);

#[no_mangle]
pub extern "sysv64" fn kernel_main_new_stack (fb_config: &FrameBufferConfig, memory_map: &MemoryMap, rsdp: u64) -> ! {
    let graphic = unsafe { Graphic::init(*fb_config) };
    graphic.clear();

    unsafe { segment::init() };
    unsafe { BitMapMemoryManager::init(memory_map) };
    unsafe { paging::init() };
    unsafe { interrupts::init() };

    println!("This is Rusmikan");
    println!("1 + 2 = {}", 1 + 2);
    // x86_64::instructions::interrupts::int3();

    unsafe {
        let addr = BITMAP_MEMORY_MANAGER.allocate(4).unwrap();
        BITMAP_MEMORY_MANAGER.free(addr, 4);
    }

    //unsafe {
    //    for i in 0..25 {
    //        start_lapic_timer();
    //        print!("Line {} LAPIC Timer elapsed : ", i);
    //        println!("{}", lapic_timer_elapsed());
    //        stop_lapic_timer();
    //    }
    //}

    serial_println!("System Info");
    list_pci_devices();

    let mm = memory_map.descriptors();
    for d in mm {
        serial_println!("{:?}", d);
    }

    serial_println!("RSDP: {}", rsdp);
    // panic!();
    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
