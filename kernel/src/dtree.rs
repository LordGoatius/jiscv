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

pub fn parse(header: *const DeviceTreeHeader) -> DeviceTree {
    assert_eq!(unsafe { *header }.magic, 0xd00dfeed_u32.to_be());
    let parser = DeviceTreeParser::from(header);
    parser.parse()
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

#[derive(Debug)]
pub struct DeviceTree {
    node_list: Vec<DeviceTreeNode>,
    resvd_mem: Vec<MemResBlock>,
}

#[derive(Debug)]
pub struct DeviceTreeNode {
    name: &'static str,
    properties: Vec<DeviceTreeProperty>,
    child_node: Vec<DeviceTreeNode>,
}

#[derive(Debug)]
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
    dt_struct_slice: &'static [u8],
    dt_strings: &'static [u8],
    resrvd_mem: *const MemResBlock,
}

impl DeviceTreeParser {
    pub fn from(header_ptr: *const DeviceTreeHeader) -> DeviceTreeParser {
        unsafe {
            let header = *header_ptr;
            // To calculate byte offsets properly
            let header_ptr = header_ptr as *const u8;

            let dt_struct =
                header_ptr.add(u32::from_be(header.off_dt_struct) as usize) as *const u8;
            let dt_struct_size = u32::from_be(header.size_dt_struct) as usize;

            let dt_struct_slice = slice::from_raw_parts(dt_struct, dt_struct_size);

            let dt_strings = slice::from_raw_parts(
                header_ptr.add(u32::from_be(header.off_dt_strings) as usize) as *const u8,
                u32::from_be(header.size_dt_strings) as usize,
            );

            let resrvd_mem =
                header_ptr.add(u32::from_be(header.off_mem_resvmap) as usize) as *const MemResBlock;

            DeviceTreeParser {
                dt_struct,
                dt_struct_size,
                dt_struct_slice,
                dt_strings,
                resrvd_mem,
            }
        }
    }

    pub fn parse(&self) -> DeviceTree {
        let resvd_mem = self.parse_reserved_mem();
        let node_list = self.parse_nodes();

        DeviceTree {
            node_list,
            resvd_mem,
        }
    }

    fn parse_nodes(&self) -> Vec<DeviceTreeNode> {
        let mut nodes = Vec::new();
        let mut curr = 0;
        let iter = self.dt_struct;
        let size = self.dt_struct_size;

        loop {
            if curr >= size {
                break;
            }

            let token = unsafe {
                let val = *(iter.add(curr) as *const u32);
                FDT_TOKEN::from_u32(val).expect("DeviceTree Parse Error")
            };

            use FDT_TOKEN::*;

            // Uphold loop invariants:
            // - iter.add(curr) is aligned to 4 bytes
            // - iter.add(curr) points to a valid FDT_NODE
            // curr is advancing
            match token {
                BEGIN_NODE => {
                    let (advance, node) = self.parse_node(curr);
                    nodes.push(node);
                    curr += advance;
                    continue;
                }
                tok @ (END_NODE | PROP) => panic!("Unexpected {tok:?} in token stream"),
                NOP => {
                    curr += 4;
                    continue;
                }
                END => break,
            }
        }

        nodes
    }

    fn parse_node(&self, mut curr: usize) -> (usize, DeviceTreeNode) {
        let mut child_node = Vec::new();
        let mut properties = Vec::new();

        let iter = self.dt_struct;

        // Advance beyond the initial BEGIN_NODE
        curr += 4;

        // Parse unit name as string
        let name = unsafe {
            let start = curr;
            let slice = self.dt_struct_slice;
            while slice[curr] != 0x00 {
                curr += 1;
            }

            str::from_utf8_unchecked(&slice[start..curr])
        };
        // add the null byte to the current count
        curr += 1;

        while curr % 4 != 0 {
            curr += 1;
        }

        loop {
            assert!(curr % 4 == 0);
            let token = unsafe {
                let val = *(iter.add(curr) as *const u32);
                FDT_TOKEN::from_u32(val).expect("DeviceTree Parse Error")
            };

            use FDT_TOKEN::*;

            match token {
                BEGIN_NODE => {
                    let (new_curr, node) = self.parse_node(curr);
                    child_node.push(node);
                    curr = new_curr;
                }
                END_NODE => {
                    curr += 4;
                    break;
                }
                PROP => {
                    curr += 4;
                    let prop = unsafe { *(iter.add(curr) as *const PropData) };

                    curr += 8;

                    properties.push(self.parse_dt_property(prop, curr));

                    curr += u32::from_be(prop.len_data) as usize;

                    while curr % 4 != 0 {
                        curr += 1;
                    }
                }
                NOP => {
                    curr += 4;
                }
                tok @ END => panic!("Unexpected {tok:?} during node parsing"),
            }
        }

        let node = DeviceTreeNode {
            name,
            properties,
            child_node,
        };

        (curr, node)
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

    fn parse_dt_property(&self, prop: PropData, data: usize) -> DeviceTreeProperty {
        let name = {
            let name_off = u32::from_be(prop.name_off) as usize;
            let strings = self.dt_strings;

            let mut iter = name_off;
            while strings[iter] != 0x00 {
                iter += 1;
            }
            
            unsafe { str::from_utf8_unchecked(&strings[name_off..iter]) }
        };

        let value = unsafe {
            slice::from_raw_parts(
                self.dt_struct.add(data),
                u32::from_be(prop.len_data) as usize,
            )
        };

        DeviceTreeProperty { name, value }
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
