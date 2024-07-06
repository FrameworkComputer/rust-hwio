// SPDX-License-Identifier: MIT

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use core::arch::asm;

use core::marker::PhantomData;

use super::io::Io;

#[cfg(all(feature = "std", target_os = "freebsd"))]
use nix::ioctl_readwrite;
#[cfg(all(feature = "std", target_os = "freebsd"))]
use std::os::fd::AsRawFd;


#[cfg(all(feature = "std", target_os = "freebsd"))]
#[repr(C)]
struct IoDevPioReq {
    access: u32,
    port: u32,
    width: u32,
    val: u32,
}
#[cfg(all(feature = "std", target_os = "freebsd"))]
ioctl_readwrite!(iodev_rw, b'I', 0, IoDevPioReq);
#[cfg(all(feature = "std", target_os = "freebsd"))]
const IODEV_PIO_READ: u32 = 0;
#[cfg(all(feature = "std", target_os = "freebsd"))]
const IODEV_PIO_WRITE: u32 = 1;

#[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Mutex,
};

#[cfg(all(feature = "std", not(any(target_os = "freebsd", target_arch = "x86", target_arch = "x86_64"))))]
lazy_static::lazy_static! {
    static ref FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/port")
            .expect("failed to open /dev/port")
        );
}

#[cfg(all(feature = "std", target_os = "freebsd"))]
lazy_static::lazy_static! {
    static ref FILE: Mutex<File> = Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/io")
            .expect("failed to open /dev/io")
        );
}

#[cfg(all(feature = "std", target_os = "linux", not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[inline(always)]
pub fn port_read(port: u16, buf: &mut [u8]) {
    let mut file = FILE.lock().unwrap();
    file.seek(SeekFrom::Start(port as u64)).unwrap();
    file.read_exact(buf).unwrap();
}

#[cfg(all(feature = "std", target_os = "linux", not(any(target_arch = "x86", target_arch = "x86_64"))))]
#[inline(always)]
pub fn port_write(port: u16, buf: &[u8]) {
    let mut file = FILE.lock().unwrap();
    file.seek(SeekFrom::Start(port as u64)).unwrap();
    file.write_all(buf).unwrap();
}

#[cfg(all(feature = "std", target_os = "freebsd"))]
#[inline(always)]
pub fn port_read(port: u16, buf: &mut [u8]) {
    let mut file = FILE.lock().unwrap();
    let fd = file.as_raw_fd();

    let mut req = IoDevPioReq {
        access: IODEV_PIO_READ,
        port: port as u32,
        width: buf.len() as u32,
        val: 0,
    };
    unsafe {
        let _res = iodev_rw(fd, &mut req).unwrap();
    }

    match buf.len() {
        1 => {
            buf[0] = req.val as u8;
        },
        2 => {
            let val = u16::to_le_bytes(req.val as u16);
            buf[0] = val[0];
            buf[1] = val[1];
        },
        4 => {
            let val = u32::to_le_bytes(req.val);
            buf[0] = val[0];
            buf[1] = val[1];
            buf[2] = val[2];
            buf[3] = val[3];
        },
        _ => panic!("Unsupported"),
    }
}

#[cfg(all(feature = "std", target_os = "freebsd"))]
#[inline(always)]
pub fn port_write(port: u16, buf: &[u8]) {
    let mut file = FILE.lock().unwrap();
    let fd = file.as_raw_fd();

    let val = match buf.len() {
        1 => buf[0] as u32,
        2 => u16::from_le_bytes([buf[0], buf[1]]) as u32,
        4 => u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]),
        _ => panic!("Unsupported"),
    };

    let mut req = IoDevPioReq {
        access: IODEV_PIO_WRITE,
        port: port as u32,
        width: buf.len() as u32,
        val
    };
    unsafe {
        let _res = iodev_rw(fd, &mut req).unwrap();
    }
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

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
    #[inline(always)]
    fn read(&self) -> u8 {
        let mut buf = [0];
        port_read(self.port, &mut buf);
        buf[0]
    }

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in al, dx", out("al") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
    #[inline(always)]
    fn write(&mut self, value: u8) {
        let buf = [value];
        port_write(self.port, &buf);
    }

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
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

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
    #[inline(always)]
    fn read(&self) -> u16 {
        let mut buf = [0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u16 |
        (buf[1] as u16) << 8
    }

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in ax, dx", out("ax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
    #[inline(always)]
    fn write(&mut self, value: u16) {
        let buf = [
            value as u8,
            (value >> 8) as u8
        ];
        port_write(self.port, &buf);
    }

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn write(&mut self, value: u16) {
        unsafe {
            asm!("out dx, ax", in("ax") value, in("dx") self.port, options(nostack));
        }
    }
}

// Read/Write for doubleword PIO
impl Io for Pio<u32> {
    type Value = u32;

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
    #[inline(always)]
    fn read(&self) -> u32 {
        let mut buf = [0, 0, 0, 0];
        port_read(self.port, &mut buf);
        buf[0] as u32 |
        (buf[1] as u32) << 8 |
        (buf[2] as u32) << 16 |
        (buf[3] as u32) << 24
    }

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in eax, dx", out("eax") value, in("dx") self.port, options(nostack));
        }
        value
    }

    #[cfg(all(feature = "std", any(target_os = "freebsd", not(any(target_arch = "x86", target_arch = "x86_64")))))]
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

    #[cfg(all(not(feature = "std"), any(target_arch = "x86", target_arch = "x86_64")))]
    #[inline(always)]
    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out dx, eax", in("eax") value, in("dx") self.port, options(nostack));
        }
    }
}
