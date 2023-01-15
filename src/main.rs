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
mod acpi;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::{FrameBufferConfig,MemoryMap};
use graphics::{Graphic, Rgb};
use x86_64::{VirtAddr, structures::paging::PageTable};
use core::fmt::Write;
use console::CONSOLE;
use pci::list_pci_devices;
use mm::{BitMapMemoryManager,BITMAP_MEMORY_MANAGER};
use paging::active_level_4_table;

use crate::paging::translate_addr;

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
    serial_println!("System Info");
    let graphic = unsafe { Graphic::init(*fb_config) };
    graphic.clear();

    unsafe { segment::init() };
    unsafe { BitMapMemoryManager::init(memory_map) };
    unsafe { paging::init() };
    unsafe { acpi::init_rsdp(rsdp) };
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

    list_pci_devices();

    let mm = memory_map.descriptors();
    for d in mm {
        serial_println!("{:x?}", d);
    }

    let phys_mem_offset = VirtAddr::new(0);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };

//    for (i,entry) in l4_table.iter().enumerate() {
//        if !entry.is_unused() {
//            serial_println!("L4 Entry {}: {:?}", i, entry);
//
//            let phys = entry.frame().unwrap().start_address();
//            let virt = phys.as_u64() + phys_mem_offset.as_u64();
//            let ptr = VirtAddr::new(virt).as_mut_ptr();
//            let l3_table: &PageTable = unsafe { &*ptr };
//        }
//    }

    let addresses = [
        0xb8000,
        0x118000,
        0x201008,
        0x0100_0020_1a10,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, phys_mem_offset) };
        serial_println!("{:?} -> {:?}", virt, phys);
    }

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
