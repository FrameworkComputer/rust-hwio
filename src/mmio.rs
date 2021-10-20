use core::mem::MaybeUninit;
use core::ops::{BitAnd, BitOr, Not};
use core::ptr;

use super::io::Io;

#[repr(packed)]
pub struct Mmio<T> {
    value: T,
}

#[allow(clippy::new_without_default)]
impl<T> Mmio<T> {
    /// Create a new Mmio without initializing
    #[allow(clippy::uninit_assumed_init)]
    pub fn new() -> Self {
        Mmio {
            value: unsafe { MaybeUninit::uninit().assume_init() }
        }
    }
}

impl<T> Io for Mmio<T> where T: Copy + PartialEq + BitAnd<Output = T> + BitOr<Output = T> + Not<Output = T> {
    type Value = T;

    fn read(&self) -> T {
        unsafe { ptr::read_volatile(&self.value) }
    }

    fn write(&mut self, value: T) {
        unsafe { ptr::write_volatile(&mut self.value, value) };
    }
}
