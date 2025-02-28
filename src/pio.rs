// SPDX-License-Identifier: MIT

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::arch::asm;

use core::marker::PhantomData;

use super::io::Io;

#[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

#[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
lazy_static::lazy_static! {
    static ref FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .expect("failed to open /dev/port")
        );
}

#[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[inline(always)]
pub fn port_read(port: u16, buf: &mut [u8]) {
    let mut file = FILE.lock().unwrap();
    file.seek(SeekFrom::Start(port as u64)).unwrap();
    file.read_exact(buf).unwrap();
}

#[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[inline(always)]
pub fn port_write(port: u16, buf: &[u8]) {
    let mut file = FILE.lock().unwrap();
    file.seek(SeekFrom::Start(port as u64)).unwrap();
    file.write_all(buf).unwrap();
}

/// Generic PIO
#[derive(Copy, Clone)]
pub struct Pio<T> {
    port: u16,
    value: PhantomData<T>,
}

impl<T> Pio<T> {
    /// Create a PIO from a given port
    pub const fn new(port: u16) -> Self {
        Pio::<T> {
            port,
            value: PhantomData,
        }
    }
}

/// Read/Write for byte PIO
impl Io for Pio<u8> {
    type Value = u8;

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn read(&self) -> u8 {
        let mut buf = [0];
        port_read(self.port, &mut buf);
        buf[0]
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn write(&mut self, value: u8) {
        let buf = [value];
        port_write(self.port, &buf);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn write(&mut self, value: u8) {
        unsafe {
            asm!("out dx, al", in("al") value, in("dx") self.port, options(nostack));
        }
    }
}

/// Read/Write for word PIO
impl Io for Pio<u16> {
    type Value = u16;

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn read(&self) -> u16 {
        let mut buf = [0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u16 |
        (buf[1] as u16) << 8
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in ax, dx", out("ax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn write(&mut self, value: u16) {
        let buf = [
            value as u8,
            (value >> 8) as u8
        ];
        port_write(self.port, &buf);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn write(&mut self, value: u16) {
        unsafe {
            asm!("out dx, ax", in("ax") value, in("dx") self.port, options(nostack));
        }
    }
}

/// Read/Write for doubleword PIO
impl Io for Pio<u32> {
    type Value = u32;

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn read(&self) -> u32 {
        let mut buf = [0, 0, 0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u32 |
        (buf[1] as u32) << 8 |
        (buf[2] as u32) << 16 |
        (buf[3] as u32) << 24
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in eax, dx", out("eax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", not(any(target_arch = "x86", target_arch = "x86_64"))))]
    #[inline(always)]
    fn write(&mut self, value: u32) {
        let buf = [
            value as u8,
            (value >> 8) as u8,
            (value >> 16) as u8,
            (value >> 24) as u8
        ];
        port_write(self.port, &buf);
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[inline(always)]
    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out dx, eax", in("eax") value, in("dx") self.port, options(nostack));
        }
    }
}
