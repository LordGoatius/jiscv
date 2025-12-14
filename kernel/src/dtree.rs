use core::fmt::Display;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceTreeHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32
}

impl Display for DeviceTreeHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "DeviceTreeHeader {{")?;
        writeln!(f, "    magic:             0x{:08x}", u32::from_be(self.magic))?;
        writeln!(f, "    totalsize:         0x{:08x}", u32::from_be(self.totalsize))?;
        writeln!(f, "    off_dt_struct:     0x{:08x}", u32::from_be(self.off_dt_struct))?;
        writeln!(f, "    off_dt_strings:    0x{:08x}", u32::from_be(self.off_dt_strings))?;
        writeln!(f, "    off_mem_rsvmap:    0x{:08x}", u32::from_be(self.off_mem_rsvmap))?;
        writeln!(f, "    version:           0x{:08x}", u32::from_be(self.version))?;
        writeln!(f, "    last_comp_version: 0x{:08x}", u32::from_be(self.last_comp_version))?;
        writeln!(f, "    boot_cpuid_phys:   0x{:08x}", u32::from_be(self.boot_cpuid_phys))?;
        writeln!(f, "    size_dt_strings:   0x{:08x}", u32::from_be(self.size_dt_strings))?;
        writeln!(f, "    size_dt_struct:    0x{:08x}", u32::from_be(self.size_dt_struct))?;
        write!(f, "}}")
   }
}

