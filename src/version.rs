// yusb/src/version.rs
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

use libusb1_sys as ffi;
use std::{ffi::CStr, fmt, str};

/// A structure that describes the version of the underlying `libusb` library.
#[allow(missing_copy_implementations)]
pub struct LibraryVersion(&'static ffi::libusb_version);

impl LibraryVersion {
    /// Library major version.
    pub fn major(&self) -> u16 {
        self.0.major
    }

    /// Library minor version.
    pub fn minor(&self) -> u16 {
        self.0.minor
    }

    /// Library micro version.
    pub fn micro(&self) -> u16 {
        self.0.micro
    }

    /// Library nano version.
    pub fn nano(&self) -> u16 {
        self.0.nano
    }

    /// Library release candidate suffix string, e.g., `"-rc4"`.
    pub fn rc(&self) -> Option<&'static str> {
        let cstr = unsafe { CStr::from_ptr(self.0.rc) };

        match str::from_utf8(cstr.to_bytes()) {
            Ok(s) if !s.is_empty() => Some(s),
            _ => None,
        }
    }
}

impl fmt::Debug for LibraryVersion {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("LibraryVersion");

        debug.field("major", &self.major());
        debug.field("minor", &self.minor());
        debug.field("micro", &self.micro());
        debug.field("nano", &self.nano());
        debug.field("rc", &self.rc());

        debug.finish()
    }
}

/// Returns a structure with the version of the running libusb library.
pub fn version() -> LibraryVersion {
    let version: &'static ffi::libusb_version = unsafe { &*ffi::libusb_get_version() };
    LibraryVersion(version)
}
