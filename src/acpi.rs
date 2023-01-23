use crate::serial_println;
use bit_field::BitField;
use core::mem;
use rsdp;
use x86_64::instructions::port::Port;

const PMTIMER_FREQ: usize = 3579545;
static mut FADT_ADDR: u64 = 0;

// FIXME: want to use acpi crate with alloc
struct Rsdp {
    ptr: *const rsdp::Rsdp,
}

impl Rsdp {
    fn new(addr: u64) -> Self {
        Self {
            ptr: addr as *const rsdp::Rsdp,
        }
    }

    unsafe fn validate(&self) -> Result<(), rsdp::RsdpError> {
        (*self.ptr).validate()
    }

    unsafe fn xsdt_address(&self) -> u64 {
        (*self.ptr).xsdt_address()
    }
}

const XSDT: [u8; 4] = *b"XSDT";
const FADT: [u8; 4] = *b"FACP";

// https://docs.rs/acpi/latest/src/acpi/sdt.rs.html#100-110
// 36 bytes
#[repr(C, packed)]
struct SdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[derive(Clone, Copy)]
struct Flags(u32);

impl Flags {
    pub fn pm_timer_is_32_bit(&self) -> bool {
        self.0.get_bit(8)
    }
}

// https://docs.rs/acpi/latest/src/acpi/fadt.rs.html#31-115
// 276 bytes
#[repr(C, packed)]
struct Fadt {
    header: SdtHeader,

    _reserved: [u8; 40], // mask un-used members

    pm_timer_block: u32,

    _reserved1: [u8; 32], // mask un-used members

    flags: Flags,

    _reserved2: [u8; 160], // mask un-used members
}

pub unsafe fn init_rsdp(addr: u64) {
    let rsdp = Rsdp::new(addr);
    if rsdp.validate().is_err() {
        panic!();
    }
    let header = rsdp.xsdt_address();
    let length = (*(header as *const SdtHeader)).length;
    let sig = (*(header as *const SdtHeader)).signature;
    serial_println!("{:?}", length);
    serial_println!("{:?}", sig);
    if sig == XSDT {
        serial_println!("It is XSDT");
    }

    let num_tables = (length as usize - mem::size_of::<SdtHeader>()) / mem::size_of::<u64>();
    serial_println!("{:?}", num_tables);
    let tables_base = (header as usize + mem::size_of::<SdtHeader>()) as *const u64;
    for i in 0..num_tables {
        let addr = tables_base.add(i);
        serial_println!("{:?}, {:?}", addr, *addr);
        let sig = (*(*addr as *const SdtHeader)).signature;
        serial_println!("{:?}", sig);
        if sig == FADT {
            serial_println!("It is FADT");
            FADT_ADDR = *addr;
            break;
        }
    }
}

pub unsafe fn wait_milliseconds_with_pm_timer(msec: u32) {
    let mut timer = Port::<u32>::new((*(FADT_ADDR as *const Fadt)).pm_timer_block as u16);
    let start = timer.read();
    serial_println!("{:?}", start);
    let mut end = start.wrapping_add((PMTIMER_FREQ * msec as usize / 1000) as u32);

    let flags = (*(FADT_ADDR as *const Fadt)).flags;
    if !flags.pm_timer_is_32_bit() {
        end &= 0x00ffffff;
    }

    if end < start {
        while timer.read() >= start {}
    }
    while timer.read() < end {}
}
