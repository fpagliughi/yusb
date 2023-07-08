// yusb/src/error.rs
//
// Copyright (c) 2015, David Cuddeback
//               2019, Ilya Averyanov
//               2023, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use libc::c_int;
use libusb1_sys::constants::*;
use std::{fmt, result};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Errors returned by the `libusb` library.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Error {
    /// Input/output error.
    Io,
    /// Invalid parameter.
    InvalidParam,
    /// Access denied (insufficient permissions).
    Access,
    /// No such device (it may have been disconnected).
    NoDevice,
    /// Entity not found.
    NotFound,
    /// Resource busy.
    Busy,
    /// Operation timed out.
    Timeout,
    /// Overflow.
    Overflow,
    /// Pipe error.
    Pipe,
    /// System call interrupted (perhaps due to signal).
    Interrupted,
    /// Insufficient memory.
    NoMem,
    /// Operation not supported or unimplemented on this platform.
    NotSupported,
    /// The device returned a malformed descriptor.
    BadDescriptor,
    /// Other error.
    Other,
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
        use Error::*;
        fmt.write_str(match self {
            Io => "Input/Output error",
            InvalidParam => "Invalid parameter",
            Access => "Access denied (insufficient permissions)",
            NoDevice => "No such device (it may have been disconnected)",
            NotFound => "Entity not found",
            Busy => "Resource busy",
            Timeout => "Operation timed out",
            Overflow => "Overflow",
            Pipe => "Pipe error",
            Interrupted => "System call interrupted (perhaps due to signal)",
            NoMem => "Insufficient memory",
            NotSupported => "Operation not supported or unimplemented on this platform",
            BadDescriptor => "Malformed descriptor",
            Other => "Other error",
        })
    }
}

impl std::error::Error for Error {}

impl From<c_int> for Error {
    fn from(err: c_int) -> Self {
        match err {
            LIBUSB_ERROR_IO => Error::Io,
            LIBUSB_ERROR_INVALID_PARAM => Error::InvalidParam,
            LIBUSB_ERROR_ACCESS => Error::Access,
            LIBUSB_ERROR_NO_DEVICE => Error::NoDevice,
            LIBUSB_ERROR_NOT_FOUND => Error::NotFound,
            LIBUSB_ERROR_BUSY => Error::Busy,
            LIBUSB_ERROR_TIMEOUT => Error::Timeout,
            LIBUSB_ERROR_OVERFLOW => Error::Overflow,
            LIBUSB_ERROR_PIPE => Error::Pipe,
            LIBUSB_ERROR_INTERRUPTED => Error::Interrupted,
            LIBUSB_ERROR_NO_MEM => Error::NoMem,
            LIBUSB_ERROR_NOT_SUPPORTED => Error::NotSupported,
            _ => Error::Other,
        }
    }
}

/// A result of a function that may return a USB `Error`.
pub type Result<T> = result::Result<T, Error>;

/// Converts an integer return value into a `Result<usize>`
pub(crate) fn usb_result(res: c_int) -> Result<usize> {
    if res >= 0 {
        Ok(res as usize)
    } else {
        Err(Error::from(res))
    }
}

#[doc(hidden)]
macro_rules! try_unsafe {
    ($x:expr) => {
        match unsafe { $x } {
            0 => (),
            err => return Err($crate::Error::from(err)),
        }
    };
}
