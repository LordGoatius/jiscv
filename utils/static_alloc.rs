// Add a free list at some point as well as static arena

pub mod init {
    use core::{
        cell::Cell,
        mem::MaybeUninit,
        ops::Deref,
        sync::atomic::{AtomicU8, Ordering},
    };

    #[repr(u8)]
    pub enum InitStatus {
        Uninit = 0,
        InProgress = 1,
        Init = 2,
    }

    struct AtomicInitStatus(AtomicU8);

    pub struct InitLater<T, F = fn(*mut T)> {
        data: Cell<MaybeUninit<T>>,
        stat: AtomicInitStatus,
        init: Cell<Option<F>>,
    }

    impl<T, F> InitLater<T, F> {
        pub const unsafe fn new(f: F) -> Self {
            InitLater {
                data: Cell::new(MaybeUninit::uninit()),
                stat: AtomicInitStatus(AtomicU8::new(InitStatus::Uninit as u8)),
                init: Cell::new(Some(f))
            }
        }
    }

    impl<T, F: FnOnce(*mut T)> InitLater<T, F> {
        fn get_status(&self) -> InitStatus {
            unsafe { core::mem::transmute(self.stat.0.load(Ordering::Acquire)) }
        }

        fn set_status(&self, stat: InitStatus) {
            self.stat.0.store(stat as u8, Ordering::Release);
        }
    }

    impl<T, F: FnOnce(*mut T)> Deref for InitLater<T, F> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            if let InitStatus::Uninit = self.get_status() {
                let f = self.init.take().unwrap();
                self.set_status(InitStatus::InProgress);
                f(self.data.as_ptr().cast_init());
                self.set_status(InitStatus::InProgress);
            }

            // Should really replace with a real compare loop
            while let InitStatus::InProgress = self.get_status() {}

            unsafe { &*self.data.as_ptr().cast_init() }
        }
    }
}

pub mod arena {
    use crate::syscall::SysErr;
    use core::mem::MaybeUninit;

    pub struct Arena<T, const N: usize> {
        data: [MaybeUninit<T>; N],
        curr: usize,
    }

    impl<T, const N: usize> const Default for Arena<T, N> {
        fn default() -> Self {
            Arena {
                data: MaybeUninit::<[T; N]>::uninit().transpose(),
                curr: 0,
            }
        }
    }

    impl<T: Unpin, const N: usize> Arena<T, N> {
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

    // NOTE: Please stabilize negative impl with trait resolution that allows this ty ilysm libs and compiler team please
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
    use core::{
        marker::PhantomPinned,
        mem::MaybeUninit,
        ops::{Deref, DerefMut}
    };

    use crate::{static_alloc::init::InitLater, syscall::SysErr};

    pub struct FreeList<T, const N: usize> {
        data: [MaybeUninit<ListLink<T>>; N],
        list: Option<*mut ListLink<T>>,
    }

    pub struct ListLink<T> {
        next: Option<*mut ListLink<T>>,
        data: MaybeUninit<T>,
        _php: PhantomPinned,
    }

    impl<T> Deref for ListLink<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            // SAFETY: This library does not deref a [`ListLink`], and users
            // accessing a list like can only access a [`ListLink`] through this
            unsafe {
                self.data.assume_init_ref()
            }
        }
    }

    impl<T> DerefMut for ListLink<T> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            // SAFETY: This library does not deref a [`ListLink`], and users
            // accessing a list like can only access a [`ListLink`] through this
            unsafe {
                self.data.assume_init_mut()
            }
        }
    }

    impl<T, const N: usize> const Default for FreeList<T, N> {
        fn default() -> Self {
            Self {
                data: MaybeUninit::<[ListLink<T>; N]>::uninit().transpose(),
                list: None,
            }
        }
    }

    impl<T, const N: usize> FreeList<T, N> {
        pub fn alloc(&mut self, val: T) -> Result<&mut ListLink<T>, SysErr> {
            let next = self.list.ok_or(SysErr::OutOfMemory)?;
            self.list = unsafe{ next.read() }.next;
            unsafe {
                next.as_mut_unchecked().data.write(val);
            }
            Ok(unsafe{ next.as_mut_unchecked() })
        }

        pub fn free(&mut self, link: &mut ListLink<T>) {
            link.next = self.list;
            self.list = Some(link);
        }

        pub const unsafe fn new() -> InitLater<Self> {
            fn init<T, const N: usize>(init: *mut FreeList<T, N>) {
                // I can't imagine anybody doing this with 1 please don't. It's not allowed.
                assert!(N > 1);

                let init = unsafe {
                    *init = FreeList::default();
                    &mut *init
                };

                let mut prev = init.data[N - 1].write(ListLink {
                    next: None,
                    data: MaybeUninit::uninit(),
                    _php: PhantomPinned
                }) as *mut ListLink<T>;
                let mut index = N - 2;

                while index > 0 {
                    prev = init.data[index].write(ListLink {
                        next: Some(prev),
                        data: MaybeUninit::uninit(),
                        _php: PhantomPinned
                    });

                    index -= 1;
                }

                init.list = Some(init.data[0].as_mut_ptr())
            }

            unsafe {
                InitLater::new(init)
            }
        }
    }
}
