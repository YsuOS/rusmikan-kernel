use x86_64::instructions::port::Port;
use crate::print;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};

pub fn poll() -> ! {
    let mut kb = Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore);
    let mut port = Port::new(0x60);
    loop {
        let scancode: u8 = unsafe { port.read() };
        if let Ok(Some(event)) = kb.add_byte(scancode) {
            if let Some(key) = kb.process_keyevent(event) {
                if let DecodedKey::Unicode(character) = key {
                    print!("{}", character);
                }
            }
        }
    }
}
