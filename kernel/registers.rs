#![allow(private_bounds)]

use core::{any::type_name, fmt::Debug, marker::PhantomData, ops::{BitAnd, BitOr}};

pub struct Read;
pub struct Write;
pub struct ReadWrite;

pub trait RegType {}
impl RegType for Read {}
impl RegType for Write {}
impl RegType for ReadWrite {}

trait RegTrait: BitOr<Output = Self> + BitAnd<Output = Self> + Sized {}

impl RegTrait for u8 {}
impl RegTrait for u16 {}
impl RegTrait for u32 {}
impl RegTrait for u64 {}

/// Deriving [`core::fmt::Debug`] is safe for any types containing this. The [`core::fmt::Debug`] implementation
/// for [`Register`] does not read or write.
#[repr(transparent)]
pub struct Register<R: RegType, T: RegTrait>(T, PhantomData<R>);

impl<R: RegType, T: RegTrait> Debug for Register<R, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Register<{}, {}>", type_name::<R>(), type_name::<T>())
    }
}

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

    #[expect(unused)]
    pub fn and(&mut self, val: T) {
        self.write(self.read() & val);
    }
}

