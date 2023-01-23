use crate::serial_println;
use bit_field::BitField;
use core::fmt::Display;
use x86_64::instructions::port::Port;

const MAX_DEVICES: usize = 32;
const MAX_FUNCTIONS: usize = 8;

const INVALID_VENDOR_ID: u16 = 0xffff;

// refs.
// https://wiki.osdev.org/PCI

static mut CONFIG_ADDRESS: Port<u32> = Port::new(0x0cf8);
static mut CONFIG_DATA: Port<u32> = Port::new(0x0cfc);

const EMPTY_DEVICE: Device = Device {
    bus: 0x0,
    device: 0x0,
    function: 0x0,
};

#[derive(Copy, Clone, Debug)]
pub struct ClassCode {
    pub base: u8,
    pub sub: u8,
    pub interface: u8,
    pub revision: u8,
}

impl Display for ClassCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}",
            self.base, self.sub, self.interface, self.revision
        )
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Device {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
}

pub struct PciDevices {
    devices: [Device; 32],
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
        for device in 0..MAX_DEVICES as u8 {
            if Device::new(bus, device, 0).read_vendor_id() != INVALID_VENDOR_ID {
                self.scan_device(bus, device);
            }
        }
    }

    fn scan_device(&mut self, bus: u8, device: u8) {
        self.scan_function(bus, device, 0);
        if Device::new(bus, device, 0).is_single_function_device() {
            self.scan_function(bus, device, 0);
            return;
        }

        for function in 1..8 {
            if Device::new(bus, device, function).read_vendor_id() != INVALID_VENDOR_ID {
                self.scan_function(bus, device, function);
            }
        }
    }

    fn scan_function(&mut self, bus: u8, device: u8, function: u8) {
        let dev = Device::new(bus, device, function);
        self.add_device(dev);
        let class_code = dev.read_class_code();
        if class_code.base == 0x06 && class_code.sub == 0x04 {
            let bus_numbers = dev.read_bus_numbers();
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

impl Device {
    fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus: bus,
            device: device,
            function: function,
        }
    }
    fn make_address(self, reg: u8) -> u32 {
        let mut value = 0;
        value.set_bits(0..8, reg as u32);
        value.set_bits(8..11, self.function as u32);
        value.set_bits(11..16, self.device as u32);
        value.set_bits(16..24, self.bus as u32);
        value.set_bit(31, true);
        value
    }

    fn read(self, reg: u8) -> u32 {
        let addr = self.make_address(reg);
        unsafe {
            CONFIG_ADDRESS.write(addr);
            CONFIG_DATA.read()
        }
    }

    fn read_header_type(self) -> u8 {
        ((self.read(0x0c) >> 16) & 0xff) as u8
    }

    fn is_single_function_device(self) -> bool {
        // Check 7th bit of header_type.
        // 1: multi function device.
        // 0: single function device.
        let header_type = self.read_header_type();
        header_type & 0x80 == 0
    }

    fn read_vendor_id(self) -> u16 {
        (self.read(0x0) & 0xffff) as u16
    }

    fn read_class_code(self) -> ClassCode {
        let r = self.read(0x08);
        ClassCode {
            base: ((r >> 24) & 0xff) as u8,
            sub: ((r >> 16) & 0xff) as u8,
            interface: ((r >> 8) & 0xff) as u8,
            revision: (r & 0xff) as u8,
        }
    }

    fn read_bus_numbers(self) -> u16 {
        (self.read(0x18) & 0xffff) as u16
    }
}

pub fn list_pci_devices() {
    let pci_devices = scan_all_bus();
    for i in 0..pci_devices.count {
        let dev = pci_devices.devices[i];
        let class_code = dev.read_class_code();
        serial_println!(
            "{}:{}.{} vend {:04x}, class {}, head {:02x}",
            dev.bus,
            dev.device,
            dev.function,
            dev.read_vendor_id(),
            class_code,
            dev.read_header_type()
        );
    }
}

fn scan_all_bus() -> PciDevices {
    let mut pci_devices = PciDevices::new();
    if Device::new(0, 0, 0).is_single_function_device() {
        pci_devices.scan_bus(0);
        return pci_devices;
    }

    for function in 1..MAX_FUNCTIONS as u8 {
        if Device::new(0, 0, function).read_vendor_id() != INVALID_VENDOR_ID {
            pci_devices.scan_bus(function);
        }
    }
    pci_devices
}
