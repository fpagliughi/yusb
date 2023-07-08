// yusb/src/device_descriptor.rs
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

use crate::fields::Version;
use libusb1_sys as ffi;
use std::fmt;

/// Describes a device.
pub struct DeviceDescriptor(ffi::libusb_device_descriptor);

impl DeviceDescriptor {
    /// Returns the device's maximum supported USB version.
    pub fn usb_version(&self) -> Version {
        Version::from_bcd(self.0.bcdUSB)
    }

    /// Returns the manufacturer's version of the device.
    pub fn device_version(&self) -> Version {
        Version::from_bcd(self.0.bcdDevice)
    }

    /// Returns the index of the string descriptor that contains the manufacturer name.
    pub fn manufacturer_string_index(&self) -> Option<u8> {
        match self.0.iManufacturer {
            0 => None,
            n => Some(n),
        }
    }

    /// Returns the index of the string descriptor that contains the product name.
    pub fn product_string_index(&self) -> Option<u8> {
        match self.0.iProduct {
            0 => None,
            n => Some(n),
        }
    }

    /// Returns the index of the string descriptor that contains the device's serial number.
    pub fn serial_number_string_index(&self) -> Option<u8> {
        match self.0.iSerialNumber {
            0 => None,
            n => Some(n),
        }
    }

    /// Returns the device's class code.
    pub fn class_code(&self) -> u8 {
        self.0.bDeviceClass
    }

    /// Returns the device's sub class code.
    pub fn sub_class_code(&self) -> u8 {
        self.0.bDeviceSubClass
    }

    /// Returns the device's protocol code.
    pub fn protocol_code(&self) -> u8 {
        self.0.bDeviceProtocol
    }

    /// Returns the device's vendor ID.
    pub fn vendor_id(&self) -> u16 {
        self.0.idVendor
    }

    /// Returns the device's product ID.
    pub fn product_id(&self) -> u16 {
        self.0.idProduct
    }

    /// Returns the maximum packet size of the device's first endpoint.
    pub fn max_packet_size(&self) -> u8 {
        self.0.bMaxPacketSize0
    }

    /// Returns the number of config descriptors available for the device.
    pub fn num_configurations(&self) -> u8 {
        self.0.bNumConfigurations
    }
}

impl fmt::Debug for DeviceDescriptor {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("DeviceDescriptor");

        debug.field("bLength", &self.0.bLength);
        debug.field("bDescriptorType", &self.0.bDescriptorType);
        debug.field("bcdUSB", &self.0.bcdUSB);
        debug.field("bDeviceClass", &self.0.bDeviceClass);
        debug.field("bDeviceSubClass", &self.0.bDeviceSubClass);
        debug.field("bDeviceProtocol", &self.0.bDeviceProtocol);
        debug.field("bMaxPacketSize", &self.0.bMaxPacketSize0);
        debug.field("idVendor", &self.0.idVendor);
        debug.field("idProduct", &self.0.idProduct);
        debug.field("bcdDevice", &self.0.bcdDevice);
        debug.field("iManufacturer", &self.0.iManufacturer);
        debug.field("iProduct", &self.0.iProduct);
        debug.field("iSerialNumber", &self.0.iSerialNumber);
        debug.field("bNumConfigurations", &self.0.bNumConfigurations);

        debug.finish()
    }
}

impl From<ffi::libusb_device_descriptor> for DeviceDescriptor {
    fn from(descr: ffi::libusb_device_descriptor) -> Self {
        Self(descr)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fields::Version;

    #[test]
    fn it_has_usb_version() {
        assert_eq!(
            Version::from_bcd(0x1234),
            DeviceDescriptor::from(device_descriptor!(bcdUSB: 0x1234)).usb_version()
        );
    }

    #[test]
    fn it_has_device_version() {
        assert_eq!(
            Version::from_bcd(0x1234),
            DeviceDescriptor::from(device_descriptor!(bcdDevice: 0x1234)).device_version()
        );
    }

    #[test]
    fn it_has_manufacturer_string_index() {
        assert_eq!(
            Some(42),
            DeviceDescriptor::from(device_descriptor!(iManufacturer: 42))
                .manufacturer_string_index()
        );
    }

    #[test]
    fn it_handles_missing_manufacturer_string_index() {
        assert_eq!(
            None,
            DeviceDescriptor::from(device_descriptor!(iManufacturer: 0))
                .manufacturer_string_index()
        );
    }

    #[test]
    fn it_has_product_string_index() {
        assert_eq!(
            Some(42),
            DeviceDescriptor::from(device_descriptor!(iProduct: 42)).product_string_index()
        );
    }

    #[test]
    fn it_handles_missing_product_string_index() {
        assert_eq!(
            None,
            DeviceDescriptor::from(device_descriptor!(iProduct: 0)).product_string_index()
        );
    }

    #[test]
    fn it_has_serial_number_string_index() {
        assert_eq!(
            Some(42),
            DeviceDescriptor::from(device_descriptor!(iSerialNumber: 42))
                .serial_number_string_index()
        );
    }

    #[test]
    fn it_handles_missing_serial_number_string_index() {
        assert_eq!(
            None,
            DeviceDescriptor::from(device_descriptor!(iSerialNumber: 0))
                .serial_number_string_index()
        );
    }

    #[test]
    fn it_has_class_code() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(bDeviceClass: 42)).class_code()
        );
    }

    #[test]
    fn it_has_sub_class_code() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(bDeviceSubClass: 42)).sub_class_code()
        );
    }

    #[test]
    fn it_has_protocol_code() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(bDeviceProtocol: 42)).protocol_code()
        );
    }

    #[test]
    fn it_has_vendor_id() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(idVendor: 42)).vendor_id()
        );
    }

    #[test]
    fn it_has_product_id() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(idProduct: 42)).product_id()
        );
    }

    #[test]
    fn it_has_max_packet_size() {
        assert_eq!(
            42,
            DeviceDescriptor::from(device_descriptor!(bMaxPacketSize0: 42)).max_packet_size()
        );
    }

    #[test]
    fn it_has_num_configurations() {
        assert_eq!(
            3,
            DeviceDescriptor::from(device_descriptor!(bNumConfigurations: 3)).num_configurations()
        );
    }
}
