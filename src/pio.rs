// SPDX-License-Identifier: MIT

use core::marker::PhantomData;

use super::io::Io;

#[cfg(feature = "stable")]
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

#[cfg(feature = "stable")]
lazy_static::lazy_static! {
    static ref FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .expect("failed to open /dev/port")
        );
}

#[cfg(feature = "stable")]
#[inline(always)]
pub fn port_read(port: u16, buf: &mut [u8]) {
    let mut file = FILE.lock().unwrap();
    file.seek(SeekFrom::Start(port as u64)).unwrap();
    file.read_exact(buf).unwrap();
}

#[cfg(feature = "stable")]
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
    #[cfg(feature = "stable")]
    pub fn new(port: u16) -> Self {
        Pio::<T> {
            port,
            value: PhantomData,
        }
    }

    /// Create a PIO from a given port
    #[cfg(not(feature = "stable"))]
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

    #[cfg(feature = "stable")]
    #[inline(always)]
    fn read(&self) -> u8 {
        let mut buf = [0];
        port_read(self.port, &mut buf);
        buf[0]
    }

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(feature = "stable")]
    #[inline(always)]
    fn write(&mut self, value: u8) {
        let buf = [value];
        port_write(self.port, &buf);
    }

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
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

    #[cfg(feature = "stable")]
    #[inline(always)]
    fn read(&self) -> u16 {
        let mut buf = [0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u16 |
        (buf[1] as u16) << 8
    }

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in ax, dx", out("ax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(feature = "stable")]
    #[inline(always)]
    fn write(&mut self, value: u16) {
        let buf = [
            value as u8,
            (value >> 8) as u8
        ];
        port_write(self.port, &buf);
    }

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
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

    #[cfg(feature = "stable")]
    #[inline(always)]
    fn read(&self) -> u32 {
        let mut buf = [0, 0, 0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u32 |
        (buf[1] as u32) << 8 |
        (buf[2] as u32) << 16 |
        (buf[3] as u32) << 24
    }

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in eax, dx", out("eax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(feature = "stable")]
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

    #[cfg(all(not(feature = "stable"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out dx, eax", in("eax") value, in("dx") self.port, options(nostack));
        }
    }
}
