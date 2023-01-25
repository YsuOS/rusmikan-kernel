#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(pointer_is_aligned)]

mod acpi;
mod allocator;
mod ascii_font;
mod console;
mod frame;
mod graphics;
mod interrupts;
mod ioapic;
mod lapic;
mod paging;
mod pci;
mod segment;
mod serial;

use alloc::{boxed::Box, vec::Vec};
use console::CONSOLE;
use core::{arch::asm, fmt::Write, panic::PanicInfo};
use frame::BitMapFrameManager;
use graphics::{Graphic, Rgb};
use paging::active_level_4_table;
use pci::list_pci_devices;
use rusmikan::{FrameBufferConfig, MemoryMap};
use x86_64::{
    structures::paging::{OffsetPageTable, Translate},
    VirtAddr,
};

extern crate alloc;

const BG_COLOR: Rgb = Rgb {
    r: 241,
    g: 141,
    b: 0,
};

pub static mut JIFFIES: u64 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
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

#[derive(Debug)]
#[repr(align(16))]
struct KernelMainStack([u8; 1024 * 1024]);

#[no_mangle]
static mut KERNEL_MAIN_STACK: KernelMainStack = KernelMainStack([0; 1024 * 1024]);

#[no_mangle]
pub extern "sysv64" fn kernel_main_new_stack(
    fb_config: &FrameBufferConfig,
    memory_map: &MemoryMap,
    rsdp: u64,
) -> ! {
    serial_println!("System Info");
    BitMapFrameManager::init(memory_map);
    paging::init();

    let graphic = unsafe { Graphic::init(*fb_config) };
    graphic.clear();

    unsafe { acpi::init_rsdp(rsdp) };
    segment::init();
    unsafe { interrupts::init() };

    //unsafe { *(0xfffffffffffffff as *mut u64) = 42 };

    println!("This is Rusmikan");
    println!("1 + 2 = {}", 1 + 2);
    // x86_64::instructions::interrupts::int3();

    list_pci_devices();

    let mm = memory_map.descriptors();
    for d in mm {
        serial_println!("{:x?}", d);
    }

    let phys_mem_offset = VirtAddr::new(0);
    let l4_table = unsafe { active_level_4_table(phys_mem_offset) };
    let mapper = unsafe { OffsetPageTable::new(l4_table, phys_mem_offset) };

    // use x86_64::structures::paging::PageTable;
    //    for (i,entry) in l4_table.iter().enumerate() {
    //        if !entry.is_unused() {
    //            serial_println!("L4 Entry {}: {:?}", i, entry);
    //
    //            let phys = entry.frame().unwrap().start_address();
    //            let virt = phys.as_u64() + phys_mem_offset.as_u64();
    //            let ptr = VirtAddr::new(virt).as_mut_ptr();
    //            let l3_table: &PageTable = unsafe { &*ptr };
    //       }
    //    }

    let addresses = [0x0, 0xb8000, 0x201008];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        serial_println!("{:?} -> {:?}", virt, phys);
    }

    {
        let mut vec: Vec<u32> = Vec::new();
        vec.push(1);
        vec.push(2);
    }

    {
        let x = Box::new([0u8; 1024]);
        let y = Box::new([0u8; 4096]);
        let z = Box::new([0u8; 1024]);
        serial_println!("{:p}", x);
        serial_println!("{:p}", y);
        serial_println!("{:p}", z);
    }

    //{
    //    let x = Box::new([0u8; 8192]);
    //}

    // panic!();
    loop {
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
fn _print(args: core::fmt::Arguments) {
    CONSOLE.lock().write_fmt(args).unwrap();
}
