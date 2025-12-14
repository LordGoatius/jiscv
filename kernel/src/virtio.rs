use spin::lazy::Lazy;

const SECTOR_SIZE: usize = 512;
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
const VIRTIO_STATUS_ACK: usize = 1;
const VIRTIO_STATUS_DRIVER: usize = 2;
const VIRTIO_STATUS_DRIVER_OK: usize = 4;
const VIRTIO_STATUS_FEAT_OK: usize = 8;
const VIRTQ_DESC_F_NEXT: usize = 1;
const VIRTQ_DESC_F_WRITE: usize = 2;
const VIRTQ_AVAIL_F_NO_INTERRUPT: usize = 1;
const VIRTIO_BLK_T_IN: usize = 0;
const VIRTIO_BLK_T_OUT: usize = 1;

// All of these MUST have no padding (using a 64 bit ISA)

static mut BLK_REQUEST_VQ: Lazy<*mut VirtioVirtualQueue> = Lazy::new(init_virtio_driver);
// These three must be initalized by the initalizer for `BLK_REQUEST_VQ`
static mut BLK_REQ: Lazy<*mut VirtioBlockReq> = Lazy::new(|| todo!());
static mut BLK_REQ_PADDR: Lazy<*mut u8> = Lazy::new(|| todo!());
static mut BLK_CAPACITY: Lazy<u64> = Lazy::new(|| todo!());

pub fn init_virtio_driver() -> *mut VirtioVirtualQueue {
    todo!()
}

#[repr(packed)]
struct VirtQueueDesc {
    add: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(packed)]
struct VirtQueueAvailableRing {
    flags: u16,
    index: u16,
    ringbuf: [u16; VIRTQ_ENTRY_NUM],
}

#[repr(packed)]
struct VirtQueueUsedElemnts {
    id: u32,
    len: u32,
}

#[repr(C, align(4096))]
struct VirtQueueUsedRing {
    flags: u32,
    index: u32,
    ring: [VirtQueueUsedElemnts; VIRTQ_ENTRY_NUM],
}

#[repr(C)]
struct VirtioVirtualQueue {
    descs: [VirtQueueDesc; VIRTQ_ENTRY_NUM],
    available: VirtQueueAvailableRing,
    used: VirtQueueUsedRing,
    size_index: usize,
    used_index: *mut u16,
    last_index: u16,
}

#[repr(packed)]
struct VirtioBlockReq {
    ty: u32,
    reserved: u32,
    sector: u64,
    data: [u8; 512],
    status: u8,
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

fn virtio_write_64(val: u64, offset: usize) {
    unsafe {
        VIRTIO_BLK_PADDR
            .add(offset)
            .cast::<u64>()
            .write_volatile(val);
    }
}

fn virtio_write_or_32(val: u32, offset: usize) {
    virtio_write_32(virtio_read_32(offset) | val, offset);
}
