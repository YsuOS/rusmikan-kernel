#![no_std]
#![no_main]

mod graphics;
mod font;
mod ascii_font;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::FrameBufferConfig;
use graphics::{PixelWriter, Rgb, RGBResv8BitPerColorPixelWriter, BGRResv8BitPerColorPixelWriter};
use font::write_string;
use arrayvec::ArrayString;
use core::fmt::Write;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[no_mangle]
pub extern "sysv64" fn kernel_main (mut fb_config: FrameBufferConfig) -> ! {
    let vert = fb_config.vertical_resolution;
    let hori = fb_config.horizontal_resolution;

    let rgb = Rgb {
        r: 241,
        g: 141,
        b: 0,
    };

    let pixel_writer: &dyn PixelWriter = match fb_config.pixel_format {
        rusmikan::PixelFormat::PixelRGBResv8BitPerColor => &RGBResv8BitPerColorPixelWriter,
        rusmikan::PixelFormat::PixelBGRResv8BitPerColor => &BGRResv8BitPerColorPixelWriter,
    };
 
    for y in 0..vert {
        for x in 0..hori {
            unsafe {
                pixel_writer.write(&mut fb_config, x, y, rgb);
            }
        }
    }

    write_string(pixel_writer, &mut fb_config, 0, 0, "A", Rgb {r: 0, g: 0, b: 255});
    write_string(pixel_writer, &mut fb_config, 0, 16, "Hello World!", Rgb {r: 0, g: 0, b: 255});

    let mut buf = ArrayString::<128>::new();
    write!(&mut buf, "1 + 2 = {}", 1 + 2).unwrap();
    write_string(pixel_writer, &mut fb_config, 0, 32, &buf, Rgb {r: 0, g: 0, b: 255});

    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

