#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[no_mangle]
pub extern "sysv64" fn kernel_main () {
    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

