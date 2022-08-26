#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod graphics;
mod ascii_font;
mod console;
mod pci;
mod interrupts;
mod segment;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::{FrameBufferConfig,MemoryMap};
use graphics::{Graphic, Rgb};
use core::fmt::Write;
use console::CONSOLE;
use pci::list_pci_devices;
use interrupts::init_idt;

const BG_COLOR: Rgb = Rgb { r: 241, g:141, b:0 };

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[repr(align(16))]
struct KernelMainStack([u8; 1024 * 1024]);

#[no_mangle]
static mut KERNEL_MAIN_STACK: KernelMainStack = KernelMainStack([0; 1024 * 1024]);

#[no_mangle]
pub extern "sysv64" fn kernel_main_new_stack (fb_config: &FrameBufferConfig, memory_map: &MemoryMap) -> ! {
    unsafe { segment::init() };

    let graphic = unsafe { Graphic::init(*fb_config) };
    graphic.clear();

    //graphic.write_string(0, 16, "Hello World!", Rgb {r: 0, g: 0, b: 255});
    
    //CONSOLE.lock().put_string(graphic, "\
    //    line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n\
    //    line 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\nline 20\n\
    //    line 21\nline 22\nline 23\nline 24\nline 25\nline 26\nline 27\n");
    print!("Hello");
    println!(" World!");
    println!("This is Rusmikan");
    println!("1 + 2 = {}", 1 + 2);

    list_pci_devices();

    init_idt();
    x86_64::instructions::interrupts::int3();

    let mm = memory_map.descriptors();
    for d in mm {
        println!("{:?}", d);
    }

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
