#![allow(private_bounds)]

use core::{marker::PhantomData, ops::BitOr};

pub struct Read;
pub struct Write;
pub struct ReadWrite;

pub trait RegType {}
impl RegType for Read {}
impl RegType for Write {}
impl RegType for ReadWrite {}

trait RegTrait: BitOr<Output = Self> + Sized {}

impl RegTrait for u8 {}
impl RegTrait for u16 {}
impl RegTrait for u32 {}
impl RegTrait for u64 {}

#[repr(transparent)]
pub struct Register<R: RegType, T: RegTrait>(T, PhantomData<R>);

impl<T: RegTrait> Register<Read, T> {
    pub fn read(&self) -> T {
        unsafe { (self as *const Self).read_volatile().0 }
    }
}

impl<T: RegTrait> Register<Write, T> {
    pub fn write(&mut self, val: T) {
        unsafe {
            (self as *mut Self).write_volatile(Self(val, PhantomData));
        }
    }
}

impl<T: RegTrait> Register<ReadWrite, T> {
    pub fn read(&self) -> T {
        unsafe { (self as *const Self).read_volatile().0 }
    }

    pub fn write(&mut self, val: T) {
        unsafe {
            (self as *mut Self).write_volatile(Self(val, PhantomData));
        }
    }

    pub fn or(&mut self, val: T) {
        self.write(self.read() | val);
    }
}

