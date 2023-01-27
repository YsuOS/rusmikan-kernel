use acpi::{
    platform::{interrupt::Apic, PmTimer, Processor},
    AcpiHandler, AcpiTables, InterruptModel, PhysicalMapping, PlatformInfo,
};
use core::ptr::NonNull;
use spin::Once;
use x86_64::{instructions::port::Port, PhysAddr, VirtAddr};

const PMTIMER_FREQ: usize = 3579545;

static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

pub unsafe fn init(addr: usize) {
    PLATFORM_INFO.call_once(|| {
        AcpiTables::from_rsdp(KernelAcpiHandler, addr)
            .unwrap()
            .platform_info()
            .unwrap()
    });
}

pub fn wait_milliseconds_with_pm_timer(pm_timer: &PmTimer, msec: u32) {
    let mut timer = Port::<u32>::new(pm_timer.base.address as u16);
    let start = unsafe { timer.read() };
    let mut end = start.wrapping_add((PMTIMER_FREQ * msec as usize / 1000) as u32);

    if !pm_timer.supports_32bit {
        end &= 0x00ffffff;
    }

    if end < start {
        while unsafe { timer.read() } >= start {}
    }
    while unsafe { timer.read() } < end {}
}

pub fn platform_info() -> &'static PlatformInfo {
    PLATFORM_INFO.get().unwrap()
}

pub fn get_apic_info() -> &'static Apic {
    match &platform_info().interrupt_model {
        InterruptModel::Apic(apic) => apic,
        _ => panic!("Could not find APIC"),
    }
}

pub fn get_pm_timer_info() -> &'static PmTimer {
    platform_info().pm_timer.as_ref().unwrap()
}

pub fn get_bsp_info() -> Processor {
    platform_info()
        .processor_info
        .as_ref()
        .unwrap()
        .boot_processor
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
