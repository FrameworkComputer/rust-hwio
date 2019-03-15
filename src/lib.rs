#![no_std]
#![cfg_attr(any(target_arch = "x86", target_arch = "x86_64"), feature(asm))]
#![feature(const_fn)]
#![feature(core_intrinsics)]

pub use self::io::*;
pub use self::mmio::*;
pub use self::pio::*;

mod io;
mod mmio;
mod pio;
