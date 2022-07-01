#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::FrameBufferConfig;
use uefi::proto::console::gop::FrameBuffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[derive(Copy,Clone)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

#[no_mangle]
pub extern "sysv64" fn kernel_main (fb_config: FrameBufferConfig) -> ! {
    let vert = fb_config.vertical_resolution;
    let hori = fb_config.horizontal_resolution;
    let pixels_per_scan_line = fb_config.pixels_per_scan_line;
    let mut fb = fb_config.frame_buffer;

    let rgb = Rgb {
        r: 241,
        g: 141,
        b: 0,
    };

    type PixelWriter = unsafe fn(&mut FrameBuffer, usize, Rgb);
    unsafe fn write_pixel_rgb(fb: &mut FrameBuffer, pixel_base: usize, rgb: Rgb) {
        fb.write_value::<[u8;3]>(pixel_base, [rgb.r, rgb.g, rgb.b]);
    }
    unsafe fn write_pixel_bgr(fb: &mut FrameBuffer, pixel_base: usize, rgb: Rgb) {
        fb.write_value::<[u8;3]>(pixel_base, [rgb.b, rgb.g, rgb.r]);
    }
    let write_pixel: PixelWriter = match fb_config.pixel_format {
        rusmikan::PixelFormat::PixelRGBResv8BitPerColor => write_pixel_rgb,
        rusmikan::PixelFormat::PixelBGRResv8BitPerColor => write_pixel_bgr,
    };

    for y in 0..vert {
        for x in 0..hori {
            unsafe {
                write_pixel(&mut fb, (x+pixels_per_scan_line*y)*4, rgb);
            }
        }
    }

    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

