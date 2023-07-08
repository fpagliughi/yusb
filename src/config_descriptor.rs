// yusb/src/config_descriptor.rs
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

use crate::Interface;
use libusb1_sys as ffi;
use std::{fmt, slice};

/// Describes a configuration.
pub struct ConfigDescriptor(*const ffi::libusb_config_descriptor);

impl Drop for ConfigDescriptor {
    fn drop(&mut self) {
        unsafe {
            ffi::libusb_free_config_descriptor(self.0);
        }
    }
}

unsafe impl Sync for ConfigDescriptor {}
unsafe impl Send for ConfigDescriptor {}

impl ConfigDescriptor {
    /// Returns the configuration number.
    pub fn number(&self) -> u8 {
        unsafe { (*self.0).bConfigurationValue }
    }

    /// Returns the device's maximum power consumption (in milliamps) in this configuration.
    pub fn max_power(&self) -> u16 {
        unsafe { u16::from((*self.0).bMaxPower) * 2 }
    }

    /// Indicates if the device is self-powered in this configuration.
    pub fn self_powered(&self) -> bool {
        unsafe { (*self.0).bmAttributes & 0x40 != 0 }
    }

    /// Indicates if the device has remote wakeup capability in this configuration.
    pub fn remote_wakeup(&self) -> bool {
        unsafe { (*self.0).bmAttributes & 0x20 != 0 }
    }

    /// Returns the index of the string descriptor that describes the configuration.
    pub fn description_string_index(&self) -> Option<u8> {
        unsafe {
            match (*self.0).iConfiguration {
                0 => None,
                n => Some(n),
            }
        }
    }

    /// Returns the number of interfaces for this configuration.
    pub fn num_interfaces(&self) -> u8 {
        unsafe { (*self.0).bNumInterfaces }
    }

    /// Returns a collection of the configuration's interfaces.
    pub fn interfaces(&self) -> Interfaces {
        let interfaces = unsafe {
            slice::from_raw_parts((*self.0).interface, (*self.0).bNumInterfaces as usize)
        };

        Interfaces {
            iter: interfaces.iter(),
        }
    }

    /// Returns the unknown 'extra' bytes that libusb does not understand.
    pub fn extra(&self) -> &[u8] {
        unsafe {
            match (*self.0).extra_length {
                len if len > 0 => slice::from_raw_parts((*self.0).extra, len as usize),
                _ => &[],
            }
        }
    }
}

impl fmt::Debug for ConfigDescriptor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("ConfigDescriptor");

        let descriptor: &ffi::libusb_config_descriptor = unsafe { &*self.0 };

        debug.field("bLength", &descriptor.bLength);
        debug.field("bDescriptorType", &descriptor.bDescriptorType);
        debug.field("wTotalLength", &descriptor.wTotalLength);
        debug.field("bNumInterfaces", &descriptor.bNumInterfaces);
        debug.field("bConfigurationValue", &descriptor.bConfigurationValue);
        debug.field("iConfiguration", &descriptor.iConfiguration);
        debug.field("bmAttributes", &descriptor.bmAttributes);
        debug.field("bMaxPower", &descriptor.bMaxPower);
        debug.field("extra", &self.extra());

        debug.finish()
    }
}

/// Iterator over a configuration's interfaces.
pub struct Interfaces<'a> {
    iter: slice::Iter<'a, ffi::libusb_interface>,
}

impl<'a> Iterator for Interfaces<'a> {
    type Item = Interface<'a>;

    fn next(&mut self) -> Option<Interface<'a>> {
        self.iter.next().map(Interface::from)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl From<*const ffi::libusb_config_descriptor> for ConfigDescriptor {
    fn from(cfg: *const ffi::libusb_config_descriptor) -> Self {
        Self(cfg)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::ManuallyDrop;

    // The Drop trait impl calls libusb_free_config_descriptor(), which would attempt to free
    // unallocated memory for a stack-allocated config descriptor. Allocating a config descriptor
    // is not a simple malloc()/free() inside libusb. Mimicking libusb's allocation would be
    // error-prone, difficult to maintain, and provide little benefit for the tests. It's easier to
    // use mem::forget() to prevent the Drop trait impl from running. The config descriptor passed
    // as `$config` should be stack-allocated to prevent memory leaks in the test suite.
    macro_rules! with_config {
        ($name:ident : $config:expr => $body:block) => {{
            let config = $config;
            let $name = ManuallyDrop::new(ConfigDescriptor::from(&config as *const _));
            $body;
        }};
    }

    #[test]
    fn it_has_number() {
        with_config!(config: config_descriptor!(bConfigurationValue: 42) => {
            assert_eq!(42, config.number());
        });
    }

    #[test]
    fn it_has_max_power() {
        with_config!(config: config_descriptor!(bMaxPower: 21) => {
            assert_eq!(42, config.max_power());
        });
    }

    #[test]
    fn it_interprets_self_powered_bit_in_attributes() {
        with_config!(config: config_descriptor!(bmAttributes: 0b0000_0000) => {
            assert_eq!(false, config.self_powered());
        });

        with_config!(config: config_descriptor!(bmAttributes: 0b0100_0000) => {
            assert_eq!(true, config.self_powered());
        });
    }

    #[test]
    fn it_interprets_remote_wakeup_bit_in_attributes() {
        with_config!(config: config_descriptor!(bmAttributes: 0b0000_0000) => {
            assert_eq!(false, config.remote_wakeup());
        });

        with_config!(config: config_descriptor!(bmAttributes: 0b0010_0000) => {
            assert_eq!(true, config.remote_wakeup());
        });
    }

    #[test]
    fn it_has_description_string_index() {
        with_config!(config: config_descriptor!(iConfiguration: 42) => {
            assert_eq!(Some(42), config.description_string_index());
        });
    }

    #[test]
    fn it_handles_missing_description_string_index() {
        with_config!(config: config_descriptor!(iConfiguration: 0) => {
            assert_eq!(None, config.description_string_index());
        });
    }

    #[test]
    fn it_has_num_interfaces() {
        let interface1 = interface!(interface_descriptor!(bInterfaceNumber: 1));
        let interface2 = interface!(interface_descriptor!(bInterfaceNumber: 2));

        with_config!(config: config_descriptor!(interface1, interface2) => {
            assert_eq!(2, config.num_interfaces());
        });
    }

    #[test]
    fn it_has_interfaces() {
        let interface = interface!(interface_descriptor!(bInterfaceNumber: 1));

        with_config!(config: config_descriptor!(interface) => {
            let interface_numbers = config.interfaces().map(|interface| {
                interface.number()
            }).collect::<Vec<_>>();

            assert_eq!(vec![1], interface_numbers);
        });
    }

    // Successful compilation shows that the lifetime of the endpoint descriptor(s) is the same
    // as the lifetime of the config descriptor.
    #[test]
    fn it_had_interfaces_with_endpoints() {
        let endpoint1 = endpoint_descriptor!(bEndpointAddress: 0x81);
        let endpoint2 = endpoint_descriptor!(bEndpointAddress: 0x01);
        let endpoint3 = endpoint_descriptor!(bEndpointAddress: 0x02);
        let interface1 = interface!(interface_descriptor!(endpoint1, endpoint2));
        let interface2 = interface!(interface_descriptor!(endpoint3));

        with_config!(config: config_descriptor!(interface1, interface2) => {
            // Exists only to name config's lifetime.
            fn named_lifetime<'a>(config: &'a ConfigDescriptor) {
                let addresses: Vec<_> = config.interfaces().flat_map(|intf| intf.descriptors()).flat_map(|desc| desc.endpoint_descriptors()).map(|ep| ep.address()).collect();
                assert_eq!(addresses, &[0x81, 0x01, 0x02]);
                let desc: crate::InterfaceDescriptor<'a> = config.interfaces().flat_map(|intf| intf.descriptors()).next().expect("There's one interface");
                let _: crate::EndpointDescriptor<'a> = desc.endpoint_descriptors().next().expect("There's one endpoint");
            }
            named_lifetime(&*config);
        })
    }
}
