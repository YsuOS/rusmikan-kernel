#![no_std]
#![no_main]

mod graphics;
mod ascii_font;
//mod console;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::FrameBufferConfig;
use graphics::{Graphic, Rgb};
use core::fmt::Write;
use arrayvec::ArrayString;
//use console::put_string;


#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop{}
}

#[no_mangle]
pub extern "sysv64" fn kernel_main (fb_config: FrameBufferConfig) -> ! {
    let vert = fb_config.vertical_resolution;
    let hori = fb_config.horizontal_resolution;

    let rgb = Rgb {
        r: 241,
        g: 141,
        b: 0,
    };
    
    let mut graphic = Graphic::new(fb_config);
 
    for y in 0..vert {
        for x in 0..hori {
            graphic.write(x, y, rgb);
        }
    }
  
    graphic.write_string(0, 16, "Hello World!", Rgb {r: 0, g: 0, b: 255});

    let mut buf = ArrayString::<128>::new();
    write!(buf, "1 + 2 = {}", 1 + 2).unwrap();
    graphic.write_string(0, 32, &buf, Rgb {r: 0, g: 0, b: 255});

//    printk(pixel_writer, &mut fb_config, format_args!("hello world {}", "!"));

    //put_string(pixel_writer, &mut fb_config, "line 1\nline 2\n\nline 4\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\n\nline 25\nline 26\nline 27\n");

    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

//fn printk (pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, args: core::fmt::Arguments) {
//    let mut buf = ArrayString::<128>::new();
//    write!(&mut buf, "{}", args).unwrap();
//    put_string(pixel_writer, fb_config, &buf);
//}
