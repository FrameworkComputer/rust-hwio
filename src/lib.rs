// SPDX-License-Identifier: MIT

#![cfg_attr(not(feature = "stable"), no_std)]
#![cfg_attr(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")), feature(asm))]
#![cfg_attr(not(feature = "stable"), feature(const_fn_trait_bound))]

pub use self::io::*;
pub use self::mmio::*;
pub use self::pio::*;

mod io;
mod mmio;
mod pio;
