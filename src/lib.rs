// yusb/src/lib.rs
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

//! This crate provides a safe wrapper around the native `libusb` library.
//!
//! Yet another fork of a fork of a Rust [libusb](https://libusb.info/) wrapper!
//!
//! This is a fork of Ilya Averyanov's [rusb](https://crates.io/crates/rusb) crate, which itself is a fork of David Cuddeback's [libusb](https://crates.io/crates/libusb) crate.
//!
//! The initial version of this crate differs from `rusb` in a number of ways:
//!
//! - Removes the `UsbContext` trait
//!     - Consolidates `Context` and `GenericContext` types into a single, concrete [`Context`](struct.Context.html) type.
//!     - Now the generic context is just an instance of [`Context`](struct.Context.html) with a _null_ inner pointer.
//! - The `Device<T>` and `DeviceList<T>` no longer need to be generic over the `Context` type (since there is now only a single context type),
//!   and are now just [`Device`](struct.Device.html) and [`DeviceList`](struct.DeviceList.html), respectively.
//! - There is a new [`Port`](struct.Port.html) type which uniquely identified the physical USB port to which a device in the system is attached.
//!     - It is a combination of the bus number and ordered list of hub ports
//!     - This helps to uniquely identify a device when multiple ones are attached with the same VID:PID and no serial number or other distinguishing feature.
//!     - Individual ports are comparable and can be converted to/from strings that use the Linux _syspath_ format, like **2-1.4.3**.
//! - The [`Speed`](struct.Speed.html) type is updated:
//!     - It can be converted to floating-point speed in Mbps, and directly displayed as such.
//!     - It is ordered and comparable like:
//! ```text
//!     if (device.speed() < Speed::Super) { println!("Plug the device into a faster port");
//! ```
//! - Some general cleanup and modernization of the code base.
//!

// Lints
#![deny(
    missing_docs,
    missing_copy_implementations,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

pub use libusb1_sys::{self as ffi, constants};

#[cfg(unix)]
pub use crate::options::disable_device_discovery;

pub use crate::{
    config_descriptor::{ConfigDescriptor, Interfaces},
    context::{Context, LogLevel},
    device::Device,
    device_descriptor::DeviceDescriptor,
    device_handle::DeviceHandle,
    device_list::{DeviceList, Devices},
    endpoint_descriptor::EndpointDescriptor,
    error::{Error, Result},
    fields::{
        request_type, Direction, Port, Recipient, RequestType, Speed, SyncType, TransferType,
        UsageType, Version,
    },
    hotplug::{Hotplug, HotplugBuilder, Registration},
    interface_descriptor::{
        EndpointDescriptors, Interface, InterfaceDescriptor, InterfaceDescriptors,
    },
    language::{Language, PrimaryLanguage, SubLanguage},
    options::UsbOption,
    version::{version, LibraryVersion},
};

#[cfg(test)]
#[macro_use]
mod test_helpers;

#[macro_use]
mod error;
mod version;

mod context;
mod device;
mod device_handle;
mod device_list;

mod config_descriptor;
mod device_descriptor;
mod endpoint_descriptor;
mod fields;
mod hotplug;
mod interface_descriptor;
mod language;
mod options;

/// Tests whether the running `libusb` library supports capability API.
pub fn has_capability() -> bool {
    Context::global().as_raw();
    unsafe { ffi::libusb_has_capability(constants::LIBUSB_CAP_HAS_CAPABILITY) != 0 }
}

/// Tests whether the running `libusb` library supports hotplug.
pub fn has_hotplug() -> bool {
    Context::global().as_raw();
    unsafe { ffi::libusb_has_capability(constants::LIBUSB_CAP_HAS_HOTPLUG) != 0 }
}

/// Tests whether the running `libusb` library has HID access.
pub fn has_hid_access() -> bool {
    Context::global().as_raw();
    unsafe { ffi::libusb_has_capability(constants::LIBUSB_CAP_HAS_HID_ACCESS) != 0 }
}

/// Tests whether the running `libusb` library supports detaching the kernel driver.
pub fn supports_detach_kernel_driver() -> bool {
    Context::global().as_raw();
    unsafe { ffi::libusb_has_capability(constants::LIBUSB_CAP_SUPPORTS_DETACH_KERNEL_DRIVER) != 0 }
}

/// Returns a list of the current USB devices. Using global context
pub fn devices() -> Result<DeviceList> {
    Context::global().devices()
}

/// Sets the log level of a `libusb` global context.
pub fn set_log_level(level: LogLevel) {
    unsafe {
        ffi::libusb_set_debug(Context::global().as_raw(), level.as_c_int());
    }
}

/// Convenience function to open a device by its vendor ID and product ID.
/// Using global context
///
/// This function is provided as a convenience for building prototypes without having to
/// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
/// applications.
///
/// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
/// On error, or if the device could not be found, it returns `None`.
pub fn open_device_with_vid_pid(vid: u16, pid: u16) -> Option<DeviceHandle> {
    let handle =
        unsafe { ffi::libusb_open_device_with_vid_pid(Context::global().as_raw(), vid, pid) };

    if handle.is_null() {
        None
    } else {
        Some(unsafe {
            DeviceHandle::from_libusb(Context::global(), std::ptr::NonNull::new_unchecked(handle))
        })
    }
}
