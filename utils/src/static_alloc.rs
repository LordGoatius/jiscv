// Add a free list at some point as well as static arena

pub mod arena {
    use core::mem::MaybeUninit;
    use crate::syscall::SysErr;

    pub struct Arena<T: Copy, const N: usize> {
        data: [MaybeUninit<T>; N],
        curr: usize
    }

    impl<T: Copy, const N: usize> Default for Arena<T, N> {
        fn default() -> Self {
            Arena { data: [MaybeUninit::uninit(); N], curr: 0 }
        }
    }

    impl<T: Copy + Unpin, const N: usize> Arena<T, N> {
        fn insert(&mut self, val: T) -> Result<&T, SysErr> {
            if self.curr >= N {
                Err(SysErr::OutOfMemory)
            } else {
                Ok(self.data[self.curr].write(val))
            }
        }

        fn insert_mut(&mut self, val: T) -> Result<&mut T, SysErr> {
            if self.curr >= N {
                Err(SysErr::OutOfMemory)
            } else {
                Ok(self.data[self.curr].write(val))
            }
        }
    }

    // NOTE: Please stabilize negative impl with trait resolution that allows this
    // impl<T: Copy + !Unpin, const N: usize> Arena<T, N> {
    //     fn insert(&mut self, val: T) -> Result<&T, SysErr> {
    //         if self.curr >= N {
    //             Err(SysErr::OutOfMemory)
    //         } else {
    //             Ok(self.data[self.curr].write(val))
    //         }
    //     }

    //     fn insert_mut(&mut self, val: T) -> Result<&mut T, SysErr> {
    //         if self.curr >= N {
    //             Err(SysErr::OutOfMemory)
    //         } else {
    //             Ok(self.data[self.curr].write(val))
    //         }
    //     }
    // }
}

pub mod list {
    use core::{marker::PhantomPinned, mem::MaybeUninit};

    pub struct FreeList<T: Clone + Copy + 'static, const N: usize> {
        data: [ListLink<T>; N],
        list: Option<*mut ListLink<T>>
    }

    #[derive(Copy, Clone)]
    struct ListLink<T: Copy + Clone> {
        next: Option<*mut ListLink<T>>,
        data: MaybeUninit<T>,
        _php: PhantomPinned
    }
}
