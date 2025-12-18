use core::{fmt::Display, slice};

use ralloc::vec::Vec;

#[repr(u32)]
#[rustfmt::skip]
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum FDT_TOKEN {
    BEGIN_NODE = u32::to_be(0x00000001),
    END_NODE   = u32::to_be(0x00000002),
    PROP       = u32::to_be(0x00000003),
    NOP        = u32::to_be(0x00000004),
    END        = u32::to_be(0x00000009),
}

impl FDT_TOKEN {
    fn from_u32(val: u32) -> Option<Self> {
        if val == u32::to_be(0x00000001)
            || val == u32::to_be(0x00000002)
            || val == u32::to_be(0x00000003)
            || val == u32::to_be(0x00000004)
            || val == u32::to_be(0x00000009)
        {
            Some(unsafe { core::mem::transmute(val) })
        } else {
            None
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DeviceTreeHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_resvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

pub struct DeviceTree {
    node_list: Vec<DeviceTreeNode>,
    resvd_mem: Vec<MemResBlock>,
}

pub struct DeviceTreeNode {
    properties: Vec<DeviceTreeProperty>,
    child_node: Vec<DeviceTreeNode>,
}

pub struct DeviceTreeProperty {
    name: &'static str,
    value: &'static [u8],
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Reserved memory block (section of memory not to be used for memory allocation)
pub struct MemResBlock {
    pub addr: *const u8,
    pub size: usize,
}

impl MemResBlock {
    fn is_zero(&self) -> bool {
        self.addr.is_null() && self.size == 0
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
/// Struct that comes after a [`FDT_TOKEN::PROP`]
pub struct PropData {
    len_data: u32,
    name_off: u32,
}

struct DeviceTreeParser {
    dt_struct: *const u8,
    dt_struct_size: usize,
    dt_strings: &'static [u8],
    resrvd_mem: *const MemResBlock,
}

impl DeviceTreeParser {
    pub fn from(header_ptr: *const DeviceTreeHeader) -> DeviceTreeParser {
        unsafe {
            let header = *header_ptr;
            // To calculate byte offsets properly
            let header_ptr = header_ptr as *const u8;

            let dt_struct = header_ptr.add(header.off_dt_struct as usize) as *const u8;
            let dt_struct_size = header.size_dt_struct as usize;

            let dt_strings = slice::from_raw_parts(
                header_ptr.add(header.off_dt_strings as usize) as *const u8,
                header.size_dt_strings as usize,
            );

            let resrvd_mem = header_ptr.add(header.off_mem_resvmap as usize) as *const MemResBlock;

            DeviceTreeParser {
                dt_struct,
                dt_struct_size,
                dt_strings,
                resrvd_mem,
            }
        }
    }

    pub fn parse(&self) -> DeviceTree {
        let resvd_mem = self.parse_reserved_mem();

        DeviceTree {
            node_list: todo!(),
            resvd_mem,
        }
    }

    fn parse_nodes(&self) -> Vec<DeviceTreeNode> {
        let nodes = Vec::new();
        let mut curr = 0;
        let iter = self.dt_struct;
        let size = self.dt_struct_size;

        loop {
            if curr >= size {
                break;
            }

            let token = unsafe {
                FDT_TOKEN::from_u32(*(iter.add(curr) as *const u32)).expect("DeviceTree Parse Error")
            };

            use FDT_TOKEN::*;

            // Uphold loop invariants:
            // - iter.add(curr) is aligned to 4 bytes
            // - iter.add(curr) points to a valid FDT_NODE
            // curr is advancing
            match token {
                BEGIN_NODE => {
                    let advance = self.parse_node(curr);
                    curr += advance;
                    continue;
                },
                tok @ (END_NODE | PROP) => panic!("Unexpected {tok:?} in token stream"),
                NOP => {
                    curr += 4;
                    continue;
                },
                END => break,
            }
        }

        nodes
    }

    fn parse_node(&self, curr: usize) -> usize {
        todo!()
    }

    fn parse_reserved_mem(&self) -> Vec<MemResBlock> {
        let mut reserved_blocks = Vec::new();
        let mut iter = self.resrvd_mem;

        // Loop until we find a null block
        loop {
            let block = unsafe { *iter };

            if block.is_zero() {
                break;
            }

            reserved_blocks.push(block);

            unsafe {
                iter = iter.add(1);
            }
        }

        reserved_blocks
    }
}

#[rustfmt::skip]
impl Display for DeviceTreeHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "DeviceTreeHeader {{")?;
        writeln!(f, "    magic:             0x{:08x}", u32::from_be(self.magic))?;
        writeln!(f, "    totalsize:         0x{:08x}", u32::from_be(self.totalsize))?;
        writeln!(f, "    off_dt_struct:     0x{:08x}", u32::from_be(self.off_dt_struct))?;
        writeln!(f, "    off_dt_strings:    0x{:08x}", u32::from_be(self.off_dt_strings))?;
        writeln!(f, "    off_mem_resvmap:   0x{:08x}", u32::from_be(self.off_mem_resvmap))?;
        writeln!(f, "    version:           0x{:08x}", u32::from_be(self.version))?;
        writeln!(f, "    last_comp_version: 0x{:08x}", u32::from_be(self.last_comp_version))?;
        writeln!(f, "    boot_cpuid_phys:   0x{:08x}", u32::from_be(self.boot_cpuid_phys))?;
        writeln!(f, "    size_dt_strings:   0x{:08x}", u32::from_be(self.size_dt_strings))?;
        writeln!(f, "    size_dt_struct:    0x{:08x}", u32::from_be(self.size_dt_struct))?;
        write!(f, "}}")
    }
}
