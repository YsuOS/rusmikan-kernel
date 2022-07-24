#![no_std]
#![no_main]

mod graphics;
mod ascii_font;
mod console;

use core::panic::PanicInfo;
use core::arch::asm;
use rusmikan::FrameBufferConfig;
use graphics::{Graphic, Rgb};
use core::fmt::Write;
use arrayvec::ArrayString;
use console::Console;


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
  
    //graphic.write_string(0, 16, "Hello World!", Rgb {r: 0, g: 0, b: 255});
    //
    //let mut buf = ArrayString::<128>::new();
    //write!(buf, "1 + 2 = {}", 1 + 2).unwrap();
    //graphic.write_string(0, 32, &buf, Rgb {r: 0, g: 0, b: 255});

    let mut console = Console::new(graphic);
    //console.put_string("line 1\nline 2");
    console.put_string("\
        line 1\nline 2\nline 3\nline 4\nline 5\nline 6\nline 7\nline 8\nline 9\nline 10\n\
        line 11\nline 12\nline 13\nline 14\nline 15\nline 16\nline 17\nline 18\nline 19\nline 20\n\
        line 21\nline 22\nline 23\nline 24\nline 25\nline 26\nline 27\n");
//    printk(pixel_writer, &mut fb_config, format_args!("hello world {}", "!"));


    loop{
        unsafe {
            asm!("hlt");
        }
    }
}

//#[macro_export]
//macro_rules! print {
//        ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
//}
//
//#[macro_export]
//macro_rules! println {
//        () => ($crate::print!("\n"));
//        ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
//}
//
//#[doc(hidden)]
//pub fn _print(args: core::fmt::Arguments) {
//        let console = Console::new();
//        use core::fmt::Write;
//            WRITER.lock().write_fmt(args).unwrap();
//}

//fn printk (pixel_writer: &dyn PixelWriter, fb_config: &mut FrameBufferConfig, args: core::fmt::Arguments) {
//    let mut buf = ArrayString::<128>::new();
//    write!(&mut buf, "{}", args).unwrap();
//    put_string(pixel_writer, fb_config, &buf);
//}
