use rusmikan::FrameBufferConfig;

#[derive(Copy,Clone)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct RGBResv8BitPerColorPixelWriter;
pub struct BGRResv8BitPerColorPixelWriter;

pub trait PixelWriter {
    unsafe fn write(&self, fb_config: &mut FrameBufferConfig, x: usize, y: usize, rgb: Rgb);
    unsafe fn write_pixel(&self, fb_config: &mut FrameBufferConfig, x: usize, y: usize, rgb: [u8;3]) {
        let pixels_per_scan_line = fb_config.pixels_per_scan_line;
        let fb = &mut fb_config.frame_buffer;
        fb.write_value((x+pixels_per_scan_line*y)*4, rgb);
    }
}

impl PixelWriter for RGBResv8BitPerColorPixelWriter {
    unsafe fn write(&self, fb_config: &mut FrameBufferConfig, x: usize, y: usize, rgb: Rgb) {
        self.write_pixel(fb_config, x, y, [rgb.r, rgb.g, rgb.b]);
    }
}

impl PixelWriter for BGRResv8BitPerColorPixelWriter {
    unsafe fn write(&self, fb_config: &mut FrameBufferConfig, x: usize, y: usize, rgb: Rgb) {
        self.write_pixel(fb_config, x, y, [rgb.b, rgb.g, rgb.r]);
    }
}
