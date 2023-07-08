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

use crate::{
    context::Context,
    device::{self, Device},
    Error, Result,
};
use libc::c_int;
use libusb1_sys as ffi;
use std::{mem, slice};

/// A list of detected USB devices.
pub struct DeviceList {
    context: Context,
    list: *const *mut ffi::libusb_device,
    len: usize,
}

impl Drop for DeviceList {
    /// Frees the device list.
    fn drop(&mut self) {
        unsafe {
            ffi::libusb_free_device_list(self.list, 1);
        }
    }
}

impl DeviceList {
    /// Create a new device list using the global context.
    pub fn new() -> Result<Self> {
        Self::new_with_context(Context::default())
    }

    /// Create a new device list using the specified context.
    pub fn new_with_context(context: Context) -> Result<Self> {
        let mut list = mem::MaybeUninit::<*const *mut ffi::libusb_device>::uninit();

        let len = unsafe { ffi::libusb_get_device_list(context.as_raw(), list.as_mut_ptr()) };
        if len < 0 {
            return Err(Error::from(len as c_int));
        }
        let list = unsafe { list.assume_init() };

        Ok(DeviceList {
            context,
            list,
            len: len as usize,
        })
    }

    /// Returns the number of devices in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the list is empty, else returns false.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns an iterator over the devices in the list.
    ///
    /// The iterator yields a sequence of `Device` objects.
    pub fn iter(&self) -> Devices {
        Devices {
            context: self.context.clone(),
            devices: unsafe { slice::from_raw_parts(self.list, self.len) },
            index: 0,
        }
    }
}

/// Iterator over detected USB devices.
pub struct Devices<'a> {
    context: Context,
    devices: &'a [*mut ffi::libusb_device],
    index: usize,
}

impl<'a> Iterator for Devices<'a> {
    type Item = Device;

    fn next(&mut self) -> Option<Device> {
        if self.index < self.devices.len() {
            let device = self.devices[self.index];

            self.index += 1;
            Some(unsafe {
                device::Device::from_libusb(
                    self.context.clone(),
                    std::ptr::NonNull::new_unchecked(device),
                )
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.devices.len() - self.index;
        (remaining, Some(remaining))
    }
}
