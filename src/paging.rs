//! This os uses Sv39 RISC-V OS virtual paging
//! An Sv39 Virtual Address contains 3 9 bit VPNs (Virtual Page Number)
//! or page table index.
//!
//! ```
//! Virtual Address
//!      9          9          9           12
//! [_________][_________][_________][____________]
//!   VPN[2]     VPN[1]     VPN[0]     Page Offset
//!
//! Physical address
//!              26                  9          9           12
//! [__________________________][_________][_________][____________]
//!            PPN[2]             PPN[1]     PPN[0]     Page Offset
//!
//! Page Table Entry
//!  N PBMT  Resvd             PPN[2]             PPN[1]     PPN[0]   RSW  D  A  G  U  X  W  R  V
//! [_][__][_______][__________________________][_________][_________][__][_][_][_][_][_][_][_][_]
//! 63  61       54                          28         19         10   8  7  6  5  4  3  2  1  0
//!
//! ```

use core::{alloc::{GlobalAlloc, Layout}, ptr::slice_from_raw_parts, slice};

use crate::alloc::GLOBAL_ALLOC;

const SATP_PPN: usize = 0;
const SATP_ASID: usize = 44;
const SATP_MODE: usize = 60;

const VADDR_VPN2: usize = 0b111111111;
const VADDR_VPN1: usize = 0b111111111;
const VADDR_VPN0: usize = 0b111111111;
const VADDR_OFFSET: usize = 0b111111111111;

const VPN2_SHIFT: usize = 30;
const VPN1_SHIFT: usize = 21;
const VPN0_SHIFT: usize = 12;

const SATP_SV39_ENABLE: usize = 0x8 << SATP_MODE;
/// "Valid" bit (entry is enabled)
const PAGE_V: usize = 1 << 0;
/// Readable
const PAGE_R: usize = 1 << 1;
/// Writable
const PAGE_W: usize = 1 << 2;
/// Executable
const PAGE_X: usize = 1 << 3;
/// User (accessible in user mode)
const PAGE_U: usize = 1 << 4;
/// Global (exist in every address space)
const PAGE_G: usize = 1 << 5;
/// Accessed (page has been accessed since last time A was set to 0)
const PAGE_A: usize = 1 << 6;
/// Dirty (page has been changed since last time D was set to 0)
const PAGE_D: usize = 1 << 7;
/// RSW
const PAGE_RSW: usize = 0b11 << 8;
///PPN0
const PAGE_PPN0: usize = 0b111111111 << 10;
///PPN1
const PAGE_PPN1: usize = 0b111111111 << 19;
///PPN2
const PAGE_PPN2: usize = 0b11111111111111111111111111 << 28;
///RESERVED
const PAGE_RESERVED: usize = 0b1111111 << 54;
/// Must be 0
const PAGE_N: usize = 1 << 63;

/// Alignment and Size for a 4KiB page
const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_SIZE: usize = const { 2usize.pow(9) };

#[repr(transparent)]
pub struct PageTableEntry(usize);

#[repr(transparent)]
pub struct VAddr(*const ());
#[repr(transparent)]
pub struct PAddr(* const ());

#[repr(C, align(4096))]
pub struct PageTable([PageTableEntry; PAGE_TABLE_SIZE]);

impl PageTable {
    fn map_page(&mut self, vaddr: VAddr, paddr: PAddr, flags: u8) {
        let table2 = &mut self.0;
        if !vaddr.0.is_aligned_to(PAGE_SIZE) {
            // TODO: Change panic
            panic!("unaligned vaddr");
        }

        if !paddr.0.is_aligned_to(PAGE_SIZE) {
            // TODO: Change panic
            panic!("unaligned paddr");
        }

        let vaddr: usize = vaddr.0 as usize;
        let paddr: usize = paddr.0 as usize;

        let vpn2 = (vaddr >> VPN2_SHIFT) & VADDR_VPN2;
        let table1 = if (table2[vpn2].0 & PAGE_V) == 0 {
            // Create next table if it doesn't exist
            let page = unsafe { GLOBAL_ALLOC.alloc_zeroed(Layout::new::<PageTable>()) } as *mut PageTable;
            table2[vpn2] = PageTableEntry(((page as usize / PAGE_SIZE) << 10) | PAGE_V);
            unsafe {
                slice::from_raw_parts_mut(page as *mut PageTableEntry, PAGE_TABLE_SIZE)
            }
        } else {
            let page = ((table2[vpn2].0 >> 10) * PAGE_SIZE) as *const PageTableEntry;
            unsafe {
                slice::from_raw_parts_mut(page as *mut PageTableEntry, PAGE_TABLE_SIZE)
            }
        };

        let vpn1 = (vaddr >> VPN1_SHIFT) & VADDR_VPN1;
        let table0 = if (table1[vpn1].0 & PAGE_V) == 0 {
            // Create next table
            let page = unsafe { GLOBAL_ALLOC.alloc_zeroed(Layout::new::<PageTable>()) } as *mut PageTable;
            table1[vpn1] = PageTableEntry(((page as usize / PAGE_SIZE) << 10) | PAGE_V);
            unsafe {
                slice::from_raw_parts_mut(page as *mut PageTableEntry, PAGE_TABLE_SIZE)
            }
        } else {
            let page = ((table1[vpn1].0 >> 10) * PAGE_SIZE) as *const PageTableEntry;
            unsafe {
                slice::from_raw_parts_mut(page as *mut PageTableEntry, PAGE_TABLE_SIZE)
            }
        };

        let vpn0 = (vaddr >> VPN0_SHIFT) & VADDR_VPN0;
        table0[vpn0] = PageTableEntry(((paddr / PAGE_SIZE) << 10) | flags as usize | PAGE_V);
    }
}
