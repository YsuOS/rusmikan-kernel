use bit_field::BitField;
use x86_64::instructions::port::Port;
use crate::println;

const MAX_DEVICES: usize = 32;
const MAX_FUNCTIONS: usize = 8;

const INVALID_VENDOR_ID: u16 = 0xffff;

// refs.
// https://wiki.osdev.org/PCI

static mut CONFIG_ADDRESS: Port<u32> = Port::new(0x0cf8);
static mut CONFIG_DATA: Port<u32> = Port::new(0x0cfc);

const EMPTY_DEVICE: Device = Device {
    bus: 0xde,
    device: 0xad,
    function: 0xbe,
    header_type: 0xef,
    class_code: ClassCode {
        base: 0,
        sub: 0,
        interface: 0,
        revision: 0,
    },
};

#[derive(Copy, Clone, Debug)]
pub struct ClassCode {
    pub base: u8,
    pub sub: u8,
    pub interface: u8,
    pub revision: u8,
}

#[derive(Copy, Clone, Debug)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub header_type: u8,
    pub class_code: ClassCode,
}

pub struct PciDevices {
    devices: [Device; MAX_DEVICES],
    count: usize,
}

impl PciDevices {
    pub fn new() -> Self {
        Self {
            devices: [EMPTY_DEVICE; MAX_DEVICES],
            count: 0,
        }
    }

    pub fn scan_bus(&mut self, bus: u8) {
        for device in 0..32 {
            if read_vendor_id(bus, device, 0) != INVALID_VENDOR_ID {
                self.scan_device(bus, device);
            }
        }
    }

    fn scan_device(&mut self, bus: u8, device: u8) {
        self.scan_function(bus, device, 0);
        if is_single_function_device(read_header_type(bus, device, 0)) {
            return;
        }

        for function in 1..8 {
            if read_vendor_id(bus, device, function) != INVALID_VENDOR_ID {
                self.scan_function(bus, device, function);
            }
        }
    }

    fn scan_function(&mut self, bus: u8, device: u8, function: u8) {
        let header_type = read_header_type(bus, device, function);
        let class_code = read_class_code(bus, device, function);
        self.add_device(Device {
            bus, device, function, header_type, class_code,
        });
        if class_code.base == 0x06 && class_code.sub == 0x04 {
            let bus_numbers = read_bus_numbers(bus, device, function);
            let secondary_bus = ((bus_numbers >> 8) & 0xff) as u8;
            self.scan_bus(secondary_bus);
        }
    }

    fn add_device(&mut self, device: Device) {
        if self.count > MAX_DEVICES {
            return;
        }
        self.devices[self.count] = device;
        self.count += 1;
    }

}

fn make_address(bus: u8, device: u8, function: u8, reg: u8) -> u32 {
    let mut value = 0;
    value.set_bits(0..8, reg as u32);
    value.set_bits(8..11, function as u32);
    value.set_bits(11..16, device as u32);
    value.set_bits(16..24, bus as u32);
    value.set_bit(31, true);
    value
}

fn read(bus: u8, device: u8, function: u8, reg: u8) -> u32 {
    let addr = make_address(bus, device, function, reg);
    unsafe {
        CONFIG_ADDRESS.write(addr);
        CONFIG_DATA.read()
    }
}

fn read_header_type(bus: u8, device: u8, function: u8) -> u8 {
    ((read(bus, device, function, 0x0c) >> 16) & 0xff) as u8
}

fn is_single_function_device(header_type: u8) -> bool {
    // Check 7th bit of header_type.
    // 1: multi function device.
    // 0: single function device.
    header_type & 0x80 == 0 
}

fn read_vendor_id(bus: u8, device: u8, function: u8) -> u16 {
    (read(bus, device, function, 0x0) & 0xffff) as u16
}

fn read_class_code(bus: u8, device: u8, function: u8) -> ClassCode {
    let r = read(bus, device, function, 0x08);
    ClassCode {
        base: ((r >> 24) & 0xff) as u8,
        sub: ((r >> 16) & 0xff) as u8,
        interface: ((r >> 8) & 0xff) as u8,
        revision: (r & 0xff) as u8,
    }
}

fn read_bus_numbers(bus: u8, device: u8, function: u8) -> u16 {
    (read(bus, device, function, 0x18) & 0xffff) as u16
}

pub fn list_pci_devices() {
    let pci_devices = scan_all_bus();
    for i in 0..pci_devices.count {
        let dev = pci_devices.devices[i];
        let vendor_id = read_vendor_id(dev.bus, dev.device, dev.function);
        println!("{}:{}.{} vend {:04x}, class {:02x}{:02x}{:02x}{:02x}, head {:02x}",
            dev.bus, dev.device, dev.function, vendor_id, 
            dev.class_code.base, dev.class_code.sub, dev.class_code.interface, dev.class_code.revision,
            dev.header_type)
    }
}

fn scan_all_bus() -> PciDevices {
    let mut pci_devices = PciDevices::new();
    let header_type = read_header_type(0,0,0);
    if is_single_function_device(header_type) {
        pci_devices.scan_bus(0);
        return pci_devices;
    }

    for function in 1..MAX_FUNCTIONS as u8 {
        if read_vendor_id(0,0,function) != INVALID_VENDOR_ID {
            pci_devices.scan_bus(function);
        }
    }
    pci_devices
}
