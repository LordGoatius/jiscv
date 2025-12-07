use core::alloc::{GlobalAlloc, Layout};

// Do a linked list allocator next

use spin::Mutex;

#[global_allocator]
pub static GLOBAL_ALLOC: BumpAlloc = BumpAlloc::new();

struct RawBump {
    next: usize,
    end: usize,
}

pub struct BumpAlloc {
    bump: Mutex<Option<RawBump>>,
}

impl BumpAlloc {
    const fn new() -> Self {
        BumpAlloc {
            bump: Mutex::new(None),
        }
    }

    pub fn init(&self, start: *mut u8, end: *mut u8) {
        self.bump.lock().replace(RawBump {
            next: start as usize,
            end: end as usize,
        });
    }
}

unsafe impl GlobalAlloc for BumpAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let mut lock = self.bump.lock();

        let raw_bump = lock.as_mut().expect("Allocator is uninitalized");

        let addr = (raw_bump.next as usize).next_multiple_of(layout.align());
        assert!(addr.saturating_add(layout.size()) <= raw_bump.end, "out of memory");

        raw_bump.next = addr + layout.size();

        addr as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
    }
}
