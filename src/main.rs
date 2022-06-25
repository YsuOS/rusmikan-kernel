#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[no_mangle]
pub extern "sysv64" fn kernel_main (fb_addr: *mut u8, fb_size: u64) -> ! {
    let fb_ptr = unsafe { core::slice::from_raw_parts_mut(fb_addr, fb_size as usize) };
    for i in 0..(fb_size/4) {
        let base = (i*4) as usize;
        fb_ptr[base] = 255;
        fb_ptr[base+1] = 0;
        fb_ptr[base+2] = 0;
    }
    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

