use uefi::boot::MemoryType;
use core::ptr::addr_of;

/// Represents kinds on memory
#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
pub enum MemmapEntryType {
    Usable = 0,         //  USABLE
    Reserved = 1,       //  ACPI_NONVOLATILE + PERSISTENT_MEMORY + RESERVED + RUNTIME_SERVICES_* + RESERVED_FOR_*
    AcpiReclaimable = 2,//  ACPI_RECLAIMABLE
    Acceptable = 3,     //  UNACEPTED
    Bootloader = 4,     //  BOOT_SERVICES_CODE + BOOT_SERVICES_DATA + LOADER_CODE + LOADER_DATA
    Mmio = 5,           //  MMIO + MMIO_PORT_SPACE
    Processor = 6,      //  PAL_CODE
    Bad = 7,            //  UNUSABLE
    KernelMemory = 8,
}


macro_rules! reserved_for_oem {
    () => {
        1879048192..2147483647
    };
}

macro_rules! reserved_for_os_loader {
    () => {
        2147483648..4294967295
    };
}

impl MemmapEntryType {

    pub fn as_int(&self) -> u32 {
        unsafe { (addr_of!(self) as *const u32).read() }
    }

    pub fn from_uefi(t: MemoryType) -> Self {
        match t {
            MemoryType::CONVENTIONAL => Self::Usable,
            MemoryType::ACPI_NON_VOLATILE => Self::Reserved,
            MemoryType::PERSISTENT_MEMORY => Self::Reserved,
            MemoryType::RESERVED => Self::Reserved,
            MemoryType::RUNTIME_SERVICES_CODE => Self::Reserved,
            MemoryType::RUNTIME_SERVICES_DATA => Self::Reserved,
            MemoryType::ACPI_RECLAIM => Self::AcpiReclaimable,
            MemoryType::UNACCEPTED => Self::Acceptable,
            MemoryType::BOOT_SERVICES_CODE => Self::Bootloader,
            MemoryType::BOOT_SERVICES_DATA => Self::Bootloader,
            MemoryType::LOADER_CODE => Self::Bootloader,
            MemoryType::LOADER_DATA => Self::Bootloader,
            MemoryType::MMIO => Self::Mmio,
            MemoryType::MMIO_PORT_SPACE => Self::Mmio,
            MemoryType::PAL_CODE => Self::Processor,
            MemoryType::UNUSABLE => Self::Bad,
            _ => {
                match t.0 {
                    reserved_for_oem!() => Self::Reserved,
                    reserved_for_os_loader!() => Self::Reserved,
                    _ => unreachable!("Memmap entry type parser error"),
                }
            }
        }
    }
}

impl core::fmt::Display for MemmapEntryType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let ent = match *self {
            Self::Usable => "Usable",
            Self::Reserved => "Reserved",
            Self::AcpiReclaimable => "ACPI reclaimable",
            Self::Acceptable => "Acceptable",
            Self::Bootloader => "Bootloader memory",
            Self::Mmio => "MMIO",
            Self::Processor => "Processor memory",
            Self::Bad => "BAD memory",
            Self::KernelMemory => "Kernel memory",
            
        };
        write!(f, "{}", ent)
    }
}