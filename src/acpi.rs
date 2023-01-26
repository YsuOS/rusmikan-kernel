use crate::serial_println;
use acpi::{platform::PmTimer, AcpiHandler, AcpiTables, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::{instructions::port::Port, PhysAddr, VirtAddr};

const PMTIMER_FREQ: usize = 3579545;

pub unsafe fn init(addr: usize) -> AcpiTables<KernelAcpiHandler> {
    AcpiTables::from_rsdp(KernelAcpiHandler, addr).unwrap()
}

pub fn wait_milliseconds_with_pm_timer(pm_timer: PmTimer, msec: u32) {
    let mut timer = Port::<u32>::new(pm_timer.base.address as u16);
    let start = unsafe { timer.read() };
    serial_println!("{:?}", start);
    let mut end = start.wrapping_add((PMTIMER_FREQ * msec as usize / 1000) as u32);

    if !pm_timer.supports_32bit {
        end &= 0x00ffffff;
    }

    if end < start {
        while unsafe { timer.read() } >= start {}
    }
    while unsafe { timer.read() } < end {}
}

#[derive(Clone)]
pub struct KernelAcpiHandler;

impl AcpiHandler for KernelAcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> acpi::PhysicalMapping<Self, T> {
        let ptr = VirtAddr::new(PhysAddr::new(physical_address as u64).as_u64()).as_mut_ptr();
        PhysicalMapping::new(
            physical_address,
            NonNull::new(ptr).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(_region: &acpi::PhysicalMapping<Self, T>) {}
}
