// yusb/src/context.rs
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

use libc::timeval;
use once_cell::sync::Lazy;
use std::{mem, os::raw::c_int, ptr, sync::Arc, time::Duration};

#[cfg(windows)]
use std::os::raw::c_long;

#[cfg(unix)]
use std::os::unix::io::RawFd;

use crate::{
    device_handle::DeviceHandle,
    device_list::DeviceList,
    hotplug::{Hotplug, HotplugBuilder, Registration},
    Error, Result,
};
use libusb1_sys::{self as ffi, constants::*};

#[cfg(windows)]
type Seconds = c_long;
#[cfg(windows)]
type MicroSeconds = c_long;

#[cfg(not(windows))]
type Seconds = libc::time_t;
#[cfg(not(windows))]
type MicroSeconds = libc::suseconds_t;

/// A `libusb` context.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Context {
    inner: Arc<ContextInner>,
}

#[derive(Debug, Eq, PartialEq)]
struct ContextInner(*mut ffi::libusb_context);

unsafe impl Sync for ContextInner {}
unsafe impl Send for ContextInner {}

impl Drop for ContextInner {
    /// Closes the `libusb` context.
    fn drop(&mut self) {
        unsafe {
            ffi::libusb_exit(self.0);
        }
    }
}

/// The global context
pub static GLOBAL_CONTEXT: Lazy<Context> = Lazy::new(|| {
    let _ = unsafe { ffi::libusb_init(ptr::null_mut()) };
    Context {
        inner: Arc::new(ContextInner(ptr::null_mut())),
    }
});

impl Context {
    /// Opens a new `libusb` context.
    pub fn new() -> Result<Self> {
        let mut context = mem::MaybeUninit::<*mut ffi::libusb_context>::uninit();

        try_unsafe!(ffi::libusb_init(context.as_mut_ptr()));

        Ok(unsafe { Self::from_raw(context.assume_init()) })
    }

    /// Creates a new `libusb` context and sets runtime options.
    pub fn with_options(opts: &[crate::UsbOption]) -> Result<Self> {
        let mut this = Self::new()?;

        for opt in opts {
            opt.apply(&mut this)?;
        }

        Ok(this)
    }

    /// Gets the global context
    pub fn global() -> Self {
        Self::default()
    }

    /// Determines if this object is a reference the global context.
    pub fn is_global(&self) -> bool {
        self.inner.0.is_null()
    }

    /// Creates yusb Context from existing libusb context.
    /// Note: This transfers ownership of the context to Rust.
    /// # Safety
    /// This is unsafe because it does not check if the context is valid,
    /// so the caller must guarantee that libusb_context is created properly.
    pub unsafe fn from_raw(raw: *mut ffi::libusb_context) -> Self {
        Context {
            inner: Arc::new(ContextInner(raw)),
        }
    }

    /// Get the raw libusb_context pointer, for advanced use in unsafe code.
    pub fn as_raw(&self) -> *mut ffi::libusb_context {
        self.inner.0
    }

    /// Returns a list of the current USB devices.
    pub fn devices(&self) -> Result<DeviceList> {
        DeviceList::new_with_context(self.clone())
    }

    /// Convenience function to open a device by its vendor ID and product ID.
    ///
    /// This function is provided as a convenience for building prototypes without having to
    /// iterate a [`DeviceList`](struct.DeviceList.html). It is not meant for production
    /// applications.
    ///
    /// Returns a device handle for the first device found matching `vendor_id` and `product_id`.
    /// On error, or if the device could not be found, it returns `None`.
    pub fn open_device_with_vid_pid(
        &self,
        vendor_id: u16,
        product_id: u16,
    ) -> Option<DeviceHandle> {
        let handle =
            unsafe { ffi::libusb_open_device_with_vid_pid(self.as_raw(), vendor_id, product_id) };
        let ptr = std::ptr::NonNull::new(handle)?;
        Some(unsafe { DeviceHandle::from_libusb(self.clone(), ptr) })
    }

    /// Opens the device with a pre-opened file descriptor.
    ///
    /// This is UNIX-only and platform-specific. It is currently working with
    /// Linux/Android, but might work with other systems in the future.
    ///
    /// Note: This function does not take ownership of the specified file
    /// descriptor. The caller has the responsibility of keeping it opened for
    /// as long as the device handle.
    #[cfg(unix)]
    #[doc(alias = "libusb_wrap_sys_device")]
    pub unsafe fn open_device_with_fd(&self, fd: RawFd) -> Result<DeviceHandle> {
        let mut handle = mem::MaybeUninit::<*mut ffi::libusb_device_handle>::uninit();

        match ffi::libusb_wrap_sys_device(self.as_raw(), fd as _, handle.as_mut_ptr()) {
            0 => {
                let ptr = std::ptr::NonNull::new(handle.assume_init()).ok_or(Error::NoDevice)?;

                Ok(DeviceHandle::from_libusb(self.clone(), ptr))
            }
            err => Err(Error::from(err)),
        }
    }

    /// Sets the log level of a `libusb` for context.
    pub fn set_log_level(&mut self, level: LogLevel) {
        unsafe {
            ffi::libusb_set_debug(self.as_raw(), level.as_c_int());
        }
    }

    /// Register a callback to be called on hotplug events. The callback's
    /// [Hotplug::device_arrived] method is called when a new device is added to
    /// the bus, and [Hotplug::device_left] is called when it is removed.
    ///
    /// Devices can optionally be filtered by vendor (`vendor_id`) and device id
    /// (`product_id`).
    ///
    /// The callback will remain registered until the returned [Registration] is
    /// dropped, which can be done explicitly with [Context::unregister_callback].
    ///
    /// When handling a [Hotplug::device_arrived] event it is considered safe to call
    /// any `yusb` function that takes a [crate::Device]. It also safe to open a device and
    /// submit **asynchronous** transfers.
    /// However, most other functions that take a [DeviceHandle] are **not safe** to call.
    /// Examples of such functions are any of the synchronous API functions or
    /// the blocking functions that retrieve various USB descriptors.
    /// These functions must be used outside of the context of the [Hotplug] functions.
    #[deprecated(since = "0.9.0", note = "Use HotplugBuilder")]
    pub fn register_callback(
        &self,
        vendor_id: Option<u16>,
        product_id: Option<u16>,
        class: Option<u8>,
        callback: Box<dyn Hotplug>,
    ) -> Result<Registration> {
        let mut builder = HotplugBuilder::new();

        let mut builder = &mut builder;
        if let Some(vendor_id) = vendor_id {
            builder = builder.vendor_id(vendor_id)
        }
        if let Some(product_id) = product_id {
            builder = builder.product_id(product_id)
        }
        if let Some(class) = class {
            builder = builder.class(class)
        }

        builder.register(self.clone(), callback)
    }

    /// Unregisters the callback corresponding to the given registration. The
    /// same thing can be achieved by dropping the registration.
    pub fn unregister_callback(&self, _reg: Registration) {}

    /// Handle any pending events.
    ///
    /// If timeout less then 1 microseconds then this function will handle
    /// any already-pending events and then immediately return in non-blocking
    /// style. If timeout is [None] then function will handle any pending
    /// events in blocking mode.
    pub fn handle_events(&self, timeout: Option<Duration>) -> Result<()> {
        let n = unsafe {
            match timeout {
                Some(t) => {
                    let tv = timeval {
                        tv_sec: t.as_secs() as Seconds,
                        tv_usec: t.subsec_nanos() as MicroSeconds / 1000,
                    };
                    ffi::libusb_handle_events_timeout_completed(self.as_raw(), &tv, ptr::null_mut())
                }
                None => ffi::libusb_handle_events_completed(self.as_raw(), ptr::null_mut()),
            }
        };
        if n < 0 {
            Err(Error::from(n as c_int))
        } else {
            Ok(())
        }
    }

    /// Interrupt any active thread that is handling events (for example with
    /// [handle_events][`Self::handle_events()`]).
    #[doc(alias = "libusb_interrupt_event_handler")]
    pub fn interrupt_handle_events(&self) {
        unsafe { ffi::libusb_interrupt_event_handler(self.as_raw()) }
    }

    /// Determine the next internal timeout that libusb needs to handle.
    /// Returns Ok(None) if there are no pending timeouts.
    pub fn next_timeout(&self) -> Result<Option<Duration>> {
        let mut tv = timeval {
            tv_sec: 0,
            tv_usec: 0,
        };
        let n = unsafe { ffi::libusb_get_next_timeout(self.as_raw(), &mut tv) };

        match n {
            0 => Ok(None),
            n if n < 0 => Err(Error::from(n as c_int)),
            _ => {
                let duration = Duration::new(tv.tv_sec as _, (tv.tv_usec * 1000) as _);
                Ok(Some(duration))
            }
        }
    }
}

impl Default for Context {
    fn default() -> Self {
        GLOBAL_CONTEXT.clone()
    }
}

/////////////////////////////////////////////////////////////////////////////

/// Library logging levels.
#[derive(Clone, Copy)]
pub enum LogLevel {
    /// No messages are printed by `libusb` (default).
    None,

    /// Error messages printed to `stderr`.
    Error,

    /// Warning and error messages are printed to `stderr`.
    Warning,

    /// Informational messages are printed to `stdout`. Warnings and error
    /// messages are printed to `stderr`.
    Info,

    /// Debug and informational messages are printed to `stdout`. Warnings and
    /// error messages are printed to `stderr`.
    Debug,
}

impl LogLevel {
    pub(crate) fn as_c_int(self) -> c_int {
        match self {
            LogLevel::None => LIBUSB_LOG_LEVEL_NONE,
            LogLevel::Error => LIBUSB_LOG_LEVEL_ERROR,
            LogLevel::Warning => LIBUSB_LOG_LEVEL_WARNING,
            LogLevel::Info => LIBUSB_LOG_LEVEL_INFO,
            LogLevel::Debug => LIBUSB_LOG_LEVEL_DEBUG,
        }
    }
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn is_global_context() {
        let ctx = Context::global();
        assert!(ctx.is_global());

        let ctx = Context::new().unwrap();
        assert!(!ctx.is_global());
    }
}
