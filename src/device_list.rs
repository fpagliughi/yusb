// yusb/src/device_list.rs
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

use crate::{Context, Device, Error, Result};
use libc::c_int;
use libusb1_sys as ffi;
use std::{
    ptr::{self, NonNull},
    slice,
};

/// A list of detected USB devices.
pub struct DeviceList {
    // The context used to get the device list
    ctx: Context,
    // The devices from the context.
    // The info is guaranteed to live as long as the context
    devs: &'static [*mut ffi::libusb_device],
}

impl Drop for DeviceList {
    /// Frees the device list.
    fn drop(&mut self) {
        unsafe {
            ffi::libusb_free_device_list(self.devs.as_ptr(), 1);
        }
    }
}

impl DeviceList {
    /// Create a new device list using the global context.
    pub fn new() -> Result<Self> {
        Self::new_with_context(Context::default())
    }

    /// Create a new device list using the specified context.
    pub fn new_with_context(ctx: Context) -> Result<Self> {
        let mut list: *const *mut ffi::libusb_device = ptr::null();
        let len = unsafe { ffi::libusb_get_device_list(ctx.as_raw(), &mut list) };

        if len < 0 {
            return Err(Error::from(len as c_int));
        }
        if list.is_null() {
            return Err(Error::NotFound);
        }
        let devs = unsafe { slice::from_raw_parts(list, len as usize) };

        Ok(DeviceList { ctx, devs })
    }

    /// Gets the context used to create this device list.
    pub fn context(&self) -> Context {
        self.ctx.clone()
    }

    /// Returns the number of devices in the list.
    pub fn len(&self) -> usize {
        self.devs.len()
    }

    /// Returns true if the list is empty, else returns false.
    pub fn is_empty(&self) -> bool {
        self.devs.is_empty()
    }

    /// Gets the device at the specified index in the list.
    pub fn get(&self, idx: usize) -> Option<Device> {
        if idx >= self.devs.len() {
            return None;
        }
        let dev = self.devs[idx];
        Some(unsafe { Device::from_libusb(self.context(), NonNull::new_unchecked(dev)) })
    }

    /// Returns an iterator over the devices in the list.
    ///
    /// The iterator yields a sequence of `Device` objects.
    pub fn iter(&self) -> Devices {
        Devices { list: self, idx: 0 }
    }

    /// Converts the device list into a vector of devices.
    pub fn into_vec(self) -> Vec<Device> {
        self.iter().collect()
    }
}

/// Iterator over detected USB devices.
pub struct Devices<'a> {
    list: &'a DeviceList,
    idx: usize,
}

impl<'a> Iterator for Devices<'a> {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.list.len() {
            let dev = self.list.get(self.idx);
            self.idx += 1;
            dev
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.devs.len() - self.idx;
        (remaining, Some(remaining))
    }
}

impl IntoIterator for DeviceList {
    type Item = Device;
    type IntoIter = DeviceListIterator;

    fn into_iter(self) -> Self::IntoIter {
        DeviceListIterator { list: self, idx: 0 }
    }
}

pub struct DeviceListIterator {
    list: DeviceList,
    idx: usize,
}

impl Iterator for DeviceListIterator {
    type Item = Device;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.list.len() {
            let dev = self.list.get(self.idx);
            self.idx += 1;
            dev
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.devs.len() - self.idx;
        (remaining, Some(remaining))
    }
}
