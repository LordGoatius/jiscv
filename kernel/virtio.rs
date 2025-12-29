use core::{
    alloc::{GlobalAlloc, Layout}, marker::PhantomData, mem::offset_of, slice, sync::atomic::Ordering
};

use crate::alloc::GLOBAL_ALLOC;

struct Read;
struct Write;
struct ReadWrite;

trait RegType {}
impl RegType for Read {}
impl RegType for Write {}
impl RegType for ReadWrite {}

#[repr(transparent)]
struct Register<R: RegType>(u32, PhantomData<R>);

impl Register<Read> {
    pub fn read(&self) -> u32 {
        unsafe { (self as *const Self).read_volatile().0 }
    }
}

impl Register<Write> {
    pub fn write(&mut self, val: u32) {
        unsafe {
            (self as *mut Self).write_volatile(Self(val, PhantomData));
        }
    }
}

impl Register<ReadWrite> {
    pub fn read(&self) -> u32 {
        unsafe { (self as *const Self).read_volatile().0 }
    }

    pub fn write(&mut self, val: u32) {
        unsafe {
            (self as *mut Self).write_volatile(Self(val, PhantomData));
        }
    }

    pub fn or(&mut self, val: u32) {
        self.write(self.read() | val);
    }
}

#[repr(C, packed(4))]
pub struct VirtioDevice {
    magic_val: Register<Read>,
    version: Register<Read>,
    device_id: Register<Read>,
    vendor_id: Register<Read>,
    device_feat: Register<Read>,
    device_feat_sel: Register<Write>,
    _0: [u32; 2],
    driver_feat: Register<Write>,
    driver_feat_sel: Register<Write>,
    _1: [u32; 2],
    queue_sel: Register<Write>,
    queue_num_max: Register<Read>,
    queue_num: Register<Write>,
    legacy_queue_num_align: Register<Write>,
    legacy_queue_pfn: Register<Write>,
    queue_ready: Register<ReadWrite>,
    _3: [u32; 2],
    queue_notify: Register<Write>,
    _4: [u32; 3],
    interrupt_status: Register<Read>,
    interrupt_ack: Register<Write>,
    _5: [u32; 2],
    status: Register<ReadWrite>,
    _6: [u32; 3],
    queue_desc_low: Register<Write>,
    queue_desc_high: Register<Write>,
    _7: [u32; 2],
    queue_driver_low: Register<Write>,
    queue_driver_high: Register<Write>,
    _8: [u32; 2],
    queue_device_low: Register<Write>,
    queue_device_high: Register<Write>,
    _9: [u32; 0x15],
    config_gen: Register<Read>,
    config: Register<ReadWrite>,
}

impl VirtioDevice {
    fn init_queue(&mut self, index: u32) -> *mut VirtioVirtualQueue {
        unsafe {
            let vq: *mut VirtioVirtualQueue = GLOBAL_ALLOC
                .alloc(Layout::new::<VirtioVirtualQueue>())
                .cast::<VirtioVirtualQueue>();
            (*vq).queue_index = index;
            (*vq).used_index = &raw mut (*vq).used.index;

            self.queue_sel.write(index);
            self.queue_num.write(VIRTQ_ENTRY_NUM as u32);
            self.legacy_queue_num_align
                .write(Layout::new::<VirtioVirtualQueue>().align() as u32);
            self.legacy_queue_pfn.write(vq as u32);

            vq
        }
    }
}

pub const VIRTIO_BLK_PADDR: *mut u8 = 0x10001000 as *mut u8;
pub const SECTOR_SIZE: usize = 512;
const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: usize = 2;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

// All of these MUST have no padding (using a 64 bit ISA)

// static mut VIRTIO_DEVICE: *mut VirtioDevice = 0x10001000 as *mut VirtioDevice;
static mut VIRTIO_DEVICE: *mut VirtioDevice = VIRTIO_BLK_PADDR as *mut VirtioDevice;

// These must be initalized by the initalizer for `BLK_REQUEST_VQ`
static mut BLK_REQUEST_VQ: *mut VirtioVirtualQueue = core::ptr::null_mut();
static mut BLK_REQ: *mut VirtioBlockReq = core::ptr::null_mut();
static mut BLK_REQ_PADDR: *mut u8 = core::ptr::null_mut();
static mut BLK_CAPACITY: usize = 0;

pub fn init_virtio() {
    unsafe {
        let virtio_dev: &'static mut VirtioDevice = &mut *VIRTIO_DEVICE;
        if virtio_dev.magic_val.read() != 0x74726976 {
            panic!("virtio: invalid magic value");
        }
        if virtio_dev.version.read() != 1 {
            panic!("virtio: invalid version");
        }
        if virtio_dev.device_id.read() != VIRTIO_DEVICE_BLK as u32 {
            panic!("virtio: invalid device id");
        }

        virtio_dev.status.write(0);
        virtio_dev.status.or(VIRTIO_STATUS_ACK);
        virtio_dev.status.or(VIRTIO_STATUS_DRIVER);
        virtio_dev.status.or(VIRTIO_STATUS_FEAT_OK);

        BLK_REQUEST_VQ = virtio_dev.init_queue(0);

        virtio_dev.status.write(VIRTIO_STATUS_DRIVER_OK);

        BLK_CAPACITY = virtio_dev.config.read() as usize * SECTOR_SIZE;
        println!("virtio-blk: capacity is 0x{:x}", { BLK_CAPACITY });

        BLK_REQ_PADDR = GLOBAL_ALLOC.alloc(Layout::new::<VirtioBlockReq>());
        BLK_REQ = BLK_REQ_PADDR.cast();
    }
}

#[repr(C, packed)]
struct VirtioBlockReq {
    ty: u32,
    reserved: u32,
    sector: usize,
    data: [u8; SECTOR_SIZE as usize],
    status: u8,
}

#[repr(C, packed)]
struct VirtQueueDesc {
    addr: *mut u8,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C, packed)]
struct VirtQueueAvailableRing {
    flags: u16,
    index: u16,
    ringbuf: [u16; VIRTQ_ENTRY_NUM],
}

#[repr(C, packed)]
struct VirtQueueUsedElemnts {
    id: u32,
    len: u32,
}

#[repr(C, packed(4096))]
struct VirtQueueUsedRing {
    flags: u16,
    index: u16,
    ring: [VirtQueueUsedElemnts; VIRTQ_ENTRY_NUM],
}

#[repr(C, packed)]
pub struct VirtioVirtualQueue {
    descs: [VirtQueueDesc; VIRTQ_ENTRY_NUM],
    available: VirtQueueAvailableRing,
    used: VirtQueueUsedRing,
    queue_index: u32,
    used_index: *mut u16,
    last_index: u16,
}

// Rust style function signature
// TODO: Better idiomatic rust within function
pub fn read_disk(buf: &mut [u8], sector: usize) {
    assert!(buf.len() == SECTOR_SIZE);
    let read_cap = unsafe { BLK_CAPACITY };
    let cap = read_cap / SECTOR_SIZE;

    if sector >= cap {
        println!(
            "virtio: tried to access sector {}, but capacity is {}",
            sector, cap
        );
    }

    unsafe {
        let req = BLK_REQ;
        (*req).sector = sector;
        (*req).ty = VIRTIO_BLK_T_IN;

        let vq = BLK_REQUEST_VQ;
        let paddr = BLK_REQ_PADDR;

        (*vq).descs[0].addr = paddr;
        (*vq).descs[0].len = (size_of::<u32>() * 2 + size_of::<u64>()) as u32;
        (*vq).descs[0].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[0].next = 1;

        (*vq).descs[1].addr = paddr.add(offset_of!(VirtioBlockReq, data));
        (*vq).descs[1].len = SECTOR_SIZE as u32;
        (*vq).descs[1].flags = VIRTQ_DESC_F_NEXT | VIRTQ_DESC_F_WRITE;
        (*vq).descs[1].next = 2;

        (*vq).descs[2].addr = paddr.add(offset_of!(VirtioBlockReq, status));
        (*vq).descs[2].len = 1;
        (*vq).descs[2].flags = VIRTQ_DESC_F_WRITE;

        vq.kick(0);

        while vq.is_busy() {}

        if (*req).status != 0 {
            println!(
                "virtio: failed to access sector {}, status = {}",
                sector,
                (*req).status
            );
            return;
        }

        buf.copy_from_slice(&(*req).data);
    }
    
}

// Rust style function signature
// TODO: Better idiomatic rust within function
pub fn write_disk(buf: &[u8], sector: usize) {
    let read_cap = unsafe { BLK_CAPACITY };
    let cap = read_cap / SECTOR_SIZE;

    if sector >= cap {
        println!(
            "virtio: tried to access sector {}, but capacity is {}",
            sector, cap
        );
    }

    unsafe {
        let req = BLK_REQ;
        (*req).sector = sector;
        (*req).ty = VIRTIO_BLK_T_OUT;

        (*req)
            .data
            .copy_from_slice(buf);

        let vq = BLK_REQUEST_VQ;
        let paddr = BLK_REQ_PADDR;

        (*vq).descs[0].addr = paddr;
        (*vq).descs[0].len = (size_of::<u32>() * 2 + size_of::<u64>()) as u32;
        (*vq).descs[0].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[0].next = 1;

        (*vq).descs[1].addr = paddr.add(offset_of!(VirtioBlockReq, data));
        (*vq).descs[1].len = SECTOR_SIZE as u32;
        (*vq).descs[1].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[1].next = 2;

        (*vq).descs[2].addr = paddr.add(offset_of!(VirtioBlockReq, status));
        (*vq).descs[2].len = 1;
        (*vq).descs[2].flags = VIRTQ_DESC_F_WRITE;

        vq.kick(0);

        while vq.is_busy() {}

        if (*req).status != 0 {
            println!(
                "virtio: failed to access sector {}, status = {}",
                sector,
                (*req).status
            );
            return;
        }
    }
}

pub fn read_write_disk(buf: *mut u8, sector: usize, write: bool) {
    let read_cap = unsafe { BLK_CAPACITY };
    let cap = read_cap / SECTOR_SIZE;

    if sector >= cap {
        println!(
            "virtio: tried to access sector {}, but capacity is {}",
            sector, cap
        );
    }

    unsafe {
        let req = BLK_REQ;
        (*req).sector = sector;
        (*req).ty = if write {
            VIRTIO_BLK_T_OUT
        } else {
            VIRTIO_BLK_T_IN
        };

        if write {
            (*req)
                .data
                .copy_from_slice(slice::from_raw_parts(buf, SECTOR_SIZE as usize));
        }

        let vq = BLK_REQUEST_VQ;
        let paddr = BLK_REQ_PADDR;

        (*vq).descs[0].addr = paddr;
        (*vq).descs[0].len = (size_of::<u32>() * 2 + size_of::<u64>()) as u32;
        (*vq).descs[0].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[0].next = 1;

        (*vq).descs[1].addr = paddr.add(offset_of!(VirtioBlockReq, data));
        (*vq).descs[1].len = SECTOR_SIZE as u32;
        (*vq).descs[1].flags = VIRTQ_DESC_F_NEXT | if write { 0 } else { VIRTQ_DESC_F_WRITE };
        (*vq).descs[1].next = 2;

        (*vq).descs[2].addr = paddr.add(offset_of!(VirtioBlockReq, status));
        (*vq).descs[2].len = 1;
        (*vq).descs[2].flags = VIRTQ_DESC_F_WRITE;

        vq.kick(0);

        while vq.is_busy() {}

        if (*req).status != 0 {
            println!(
                "virtio: failed to access sector {}, status = {}",
                sector,
                (*req).status
            );
            return;
        }

        if !write {
            slice::from_raw_parts_mut(buf, SECTOR_SIZE as usize).copy_from_slice(&(*req).data);
        }
    }
}

impl VirtioVirtualQueue {
    pub fn kick(self: *mut Self, desc_ind: u16) {
        unsafe {
            (*self).available.ringbuf[(*self).available.index as usize % VIRTQ_ENTRY_NUM] =
                desc_ind;
            (*self).available.index += 1;
            core::sync::atomic::fence(Ordering::SeqCst);

            (*VIRTIO_DEVICE)
                .queue_notify
                .write((*self).queue_index as u32);

            (*self).last_index += 1;
        }
    }

    pub fn is_busy(self: *mut Self) -> bool {
        unsafe {
            self.read_volatile().last_index != self.read_volatile().used_index.read_volatile()
        }
    }
}
