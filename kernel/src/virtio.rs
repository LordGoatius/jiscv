use core::{
    alloc::{GlobalAlloc, Layout},
    marker::PhantomData,
    slice,
    sync::atomic::Ordering,
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
pub struct Register<T: Copy, R: RegType>(T, PhantomData<R>);

impl<T: Copy> Register<T, Read> {
    pub fn read(&self) -> T {
        self.0
    }
}

impl<T: Copy> Register<T, Write> {
    pub fn write(&mut self, val: T) {
        self.0 = val;
    }
}

impl<T: Copy> Register<T, ReadWrite> {
    pub fn read(&self) -> T {
        self.0
    }

    pub fn write(&mut self, val: T) {
        self.0 = val;
    }
}

#[repr(C, packed(4))]
pub struct VirtioDevice {
    magic_val: Register<u32, Read>,
    version: Register<u32, Read>,
    device_id: Register<u32, Read>,
    vendor_id: Register<u32, Read>,
    device_feat: Register<u32, Read>,
    device_feat_sel: Register<u32, Write>,
    _0: [u32; 2],
    driver_feat: Register<u32, Write>,
    driver_feat_sel: Register<u32, Write>,
    _1: [u32; 2],
    queue_sel: Register<u32, Write>,
    queue_num_max: Register<u32, Read>,
    queue_num: Register<u32, Write>,
    _2: [u32; 2],
    queue_ready: Register<u32, ReadWrite>,
    _3: [u32; 2],
    queue_notify: Register<u32, Write>,
    _4: [u32; 3],
    interrupt_status: Register<u32, Read>,
    interrupt_ack: Register<u32, Write>,
    _5: [u32; 2],
    status: Register<u32, ReadWrite>,
    _6: [u32; 3],
    queue_desc: Register<u64, Write>,
    _7: [u32; 2],
    queue_driver: Register<u64, Write>,
    _8: [u32; 2],
    queue_device: Register<u64, Write>,
    _9: [u32; 0x15],
    config_gen: Register<u32, Read>,
    config: Register<u32, ReadWrite>,
}

pub const SECTOR_SIZE: u64 = 512;
const VIRTQ_ENTRY_NUM: usize = 16;
const VIRTIO_DEVICE_BLK: usize = 2;
pub const VIRTIO_BLK_PADDR: *mut u8 = 0x10001000 as *mut u8;
const VIRTIO_REG_MAGIC: usize = 0x00;
const VIRTIO_REG_VERSION: usize = 0x04;
const VIRTIO_REG_DEVICE_ID: usize = 0x08;
const VIRTIO_REG_QUEUE_SEL: usize = 0x30;
const VIRTIO_REG_QUEUE_NUM_MAX: usize = 0x34;
const VIRTIO_REG_QUEUE_NUM: usize = 0x38;
const VIRTIO_REG_QUEUE_ALIGN: usize = 0x3c;
const VIRTIO_REG_QUEUE_PFN: usize = 0x40;
const VIRTIO_REG_QUEUE_READY: usize = 0x44;
const VIRTIO_REG_QUEUE_NOTIFY: usize = 0x50;
const VIRTIO_REG_DEVICE_STATUS: usize = 0x70;
const VIRTIO_REG_DEVICE_CONFIG: usize = 0x100;
const VIRTIO_STATUS_ACK: u32 = 1;
const VIRTIO_STATUS_DRIVER: u32 = 2;
const VIRTIO_STATUS_DRIVER_OK: u32 = 4;
const VIRTIO_STATUS_FEAT_OK: u32 = 8;
const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;
const VIRTQ_AVAIL_F_NO_INTERRUPT: usize = 1;
const VIRTIO_BLK_T_IN: u32 = 0;
const VIRTIO_BLK_T_OUT: u32 = 1;

// All of these MUST have no padding (using a 64 bit ISA)

// These must be initalized by the initalizer for `BLK_REQUEST_VQ`
static mut BLK_REQUEST_VQ: *mut VirtioVirtualQueue = core::ptr::null_mut();
static mut BLK_REQ: *mut VirtioBlockReq = core::ptr::null_mut();
static mut BLK_REQ_PADDR: *mut u8 = core::ptr::null_mut();
static mut BLK_CAPACITY: u64 = 0;

pub fn init_virtio() {
    if virtio_read_32(VIRTIO_REG_MAGIC) != 0x74726976 {
        panic!("virtio: invalid magic value");
    }
    if virtio_read_32(VIRTIO_REG_VERSION) != 1 {
        panic!("virtio: invalid version");
    }
    if virtio_read_32(VIRTIO_REG_DEVICE_ID) != VIRTIO_DEVICE_BLK as u32 {
        panic!("virtio: invalid device id");
    }

    virtio_write_32(0, VIRTIO_REG_DEVICE_STATUS);
    virtio_write_or_32(VIRTIO_STATUS_ACK, VIRTIO_REG_DEVICE_STATUS);
    virtio_write_or_32(VIRTIO_STATUS_DRIVER, VIRTIO_REG_DEVICE_STATUS);
    virtio_write_or_32(VIRTIO_STATUS_FEAT_OK, VIRTIO_REG_DEVICE_STATUS);

    unsafe {
        BLK_REQUEST_VQ = VirtioVirtualQueue::init(0);
    }

    virtio_write_32(VIRTIO_STATUS_DRIVER_OK, VIRTIO_REG_DEVICE_STATUS);

    unsafe {
        BLK_CAPACITY = virtio_read_64(VIRTIO_REG_DEVICE_CONFIG) * SECTOR_SIZE;
        println!("virtio-blk: capacity is {}", { BLK_CAPACITY });

        BLK_REQ_PADDR = GLOBAL_ALLOC.alloc(Layout::new::<VirtioBlockReq>());
        BLK_REQ = BLK_REQ_PADDR.cast();
    }
}

#[repr(C, packed)]
struct VirtioBlockReq {
    ty: u32,
    reserved: u32,
    sector: u64,
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

pub fn read_write_disk(buf: *mut u8, sector: u64, write: bool) {
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

        (*vq).descs[0].addr = BLK_REQ_PADDR;
        (*vq).descs[0].len = (size_of::<u32>() * 2 + size_of::<u64>()) as u32;
        (*vq).descs[0].flags = VIRTQ_DESC_F_NEXT;
        (*vq).descs[0].next = 1;

        // (*vq).descs[1].addr = BLK_REQ_PADDR.add(offset_of!(VirtioBlockReq, data));
        (*vq).descs[1].len = SECTOR_SIZE as u32;
        (*vq).descs[1].flags = VIRTQ_DESC_F_NEXT | if write { 0 } else { VIRTQ_DESC_F_WRITE };
        (*vq).descs[1].next = 2;

        // (*vq).descs[2].addr = BLK_REQ_PADDR.add(offset_of!(VirtioBlockReq, status));
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
    fn init(index: u32) -> *mut VirtioVirtualQueue {
        unsafe {
            let vq: *mut VirtioVirtualQueue = GLOBAL_ALLOC
                .alloc(Layout::new::<VirtioVirtualQueue>())
                .cast::<VirtioVirtualQueue>();
            (*vq).queue_index = index;
            (*vq).used_index = &raw mut (*vq).used.index;

            virtio_write_32(index, VIRTIO_REG_QUEUE_SEL);
            virtio_write_32(VIRTQ_ENTRY_NUM as u32, VIRTIO_REG_QUEUE_NUM);
            virtio_write_32(0, VIRTIO_REG_QUEUE_ALIGN);
            virtio_write_32(vq as u32, VIRTIO_REG_QUEUE_PFN);

            vq
        }
    }

    pub fn kick(self: *mut Self, desc_ind: u16) {
        unsafe {
            (*self).available.ringbuf[(*self).available.index as usize % VIRTQ_ENTRY_NUM] =
                desc_ind;
            (*self).available.index += 1;
            core::sync::atomic::fence(Ordering::SeqCst);
            virtio_write_32((*self).queue_index as u32, VIRTIO_REG_QUEUE_NOTIFY);
            (*self).last_index += 1;
        }
    }

    pub fn is_busy(self: *mut Self) -> bool {
        unsafe {
            dbg!(self.read_volatile().last_index)
                != dbg!(self.read_volatile().used_index.read_volatile())
        }
    }
}

fn virtio_read_32(offset: usize) -> u32 {
    unsafe { VIRTIO_BLK_PADDR.add(offset).cast::<u32>().read_volatile() }
}

fn virtio_read_64(offset: usize) -> u64 {
    unsafe { VIRTIO_BLK_PADDR.add(offset).cast::<u64>().read_volatile() }
}

fn virtio_write_32(val: u32, offset: usize) {
    unsafe {
        VIRTIO_BLK_PADDR
            .add(offset)
            .cast::<u32>()
            .write_volatile(val);
    }
}

fn virtio_write_or_32(val: u32, offset: usize) {
    virtio_write_32(virtio_read_32(offset) | val, offset);
}
