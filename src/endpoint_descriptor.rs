// yusb/src/endpoint_descriptor.rs
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

use crate::fields::{Direction, IsoSyncType, IsoUsageType, TransferType};
use libusb1_sys::{constants::*, libusb_endpoint_descriptor};
use std::{fmt, slice};

/// Describes an endpoint.
pub struct EndpointDescriptor<'a>(&'a libusb_endpoint_descriptor);

impl<'a> EndpointDescriptor<'a> {
    /// Returns the endpoint's address.
    pub fn address(&self) -> u8 {
        self.0.bEndpointAddress
    }

    /// Returns the endpoint number.
    pub fn number(&self) -> u8 {
        self.0.bEndpointAddress & 0x07
    }

    /// Returns the endpoint's direction.
    pub fn direction(&self) -> Direction {
        Direction::from(self.0.bEndpointAddress & LIBUSB_ENDPOINT_DIR_MASK)
    }

    /// Returns the endpoint's transfer type.
    pub fn transfer_type(&self) -> TransferType {
        TransferType::from(self.0.bmAttributes & LIBUSB_TRANSFER_TYPE_MASK)
    }

    /// Returns the isochronous endpoint's synchronisation mode.
    ///
    /// The return value of this method is only valid for isochronous endpoints.
    pub fn iso_sync_type(&self) -> IsoSyncType {
        IsoSyncType::from((self.0.bmAttributes & LIBUSB_ISO_SYNC_TYPE_MASK) >> 2)
    }

    /// Returns the isochronous endpoint's usage type.
    ///
    /// The return value of this method is only valid for isochronous endpoints.
    pub fn iso_usage_type(&self) -> IsoUsageType {
        IsoUsageType::from((self.0.bmAttributes & LIBUSB_ISO_USAGE_TYPE_MASK) >> 4)
    }

    /// Returns the endpoint's maximum packet size.
    pub fn max_packet_size(&self) -> u16 {
        self.0.wMaxPacketSize
    }

    /// Returns the endpoint's polling interval.
    pub fn interval(&self) -> u8 {
        self.0.bInterval
    }

    /// Returns the unknown 'extra' bytes that libusb does not understand.
    pub fn extra(&'a self) -> Option<&'a [u8]> {
        unsafe {
            match self.0.extra_length {
                len if len > 0 => Some(slice::from_raw_parts(self.0.extra, len as usize)),
                _ => None,
            }
        }
    }

    /// For audio devices only: return the rate at which synchronization feedback is provided.
    pub fn refresh(&self) -> u8 {
        self.0.bRefresh
    }

    /// For audio devices only: return the address if the synch endpoint.
    pub fn synch_address(&self) -> u8 {
        self.0.bSynchAddress
    }
}

impl<'a> fmt::Debug for EndpointDescriptor<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let mut debug = fmt.debug_struct("EndpointDescriptor");

        debug.field("bLength", &self.0.bLength);
        debug.field("bDescriptorType", &self.0.bDescriptorType);
        debug.field("bEndpointAddress", &self.0.bEndpointAddress);
        debug.field("bmAttributes", &self.0.bmAttributes);
        debug.field("wMaxPacketSize", &self.0.wMaxPacketSize);
        debug.field("bInterval", &self.0.bInterval);

        debug.finish()
    }
}

impl<'a> From<&'a libusb_endpoint_descriptor> for EndpointDescriptor<'a> {
    fn from(endpoint: &'a libusb_endpoint_descriptor) -> Self {
        Self(endpoint)
    }
}

#[cfg(test)]
mod test {
    #![allow(unused_qualifications)]

    use super::*;
    use crate::fields::{Direction, IsoSyncType, IsoUsageType, TransferType};

    #[test]
    fn it_interprets_number_for_output_endpoints() {
        assert_eq!(
            0,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000)).number()
        );
        assert_eq!(
            1,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0000_0001)).number()
        );
    }

    #[test]
    fn it_interprets_number_for_input_endpoints() {
        assert_eq!(
            2,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b1000_0010)).number()
        );
        assert_eq!(
            3,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b1000_0011)).number()
        );
    }

    #[test]
    fn it_ignores_reserved_bits_in_address() {
        assert_eq!(
            0,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0000_1000)).number()
        );
        assert_eq!(
            0,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0001_0000)).number()
        );
        assert_eq!(
            0,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0010_0000)).number()
        );
        assert_eq!(
            0,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0100_0000)).number()
        );
        assert_eq!(
            7,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b1111_1111)).number()
        );
    }

    #[test]
    fn it_interprets_direction_bit_in_address() {
        assert_eq!(
            Direction::Out,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b0000_0000))
                .direction()
        );
        assert_eq!(
            Direction::In,
            EndpointDescriptor::from(&endpoint_descriptor!(bEndpointAddress: 0b1000_0000))
                .direction()
        );
    }

    #[test]
    fn it_interprets_transfer_type_in_attributes() {
        assert_eq!(
            TransferType::Control,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0000))
                .transfer_type()
        );
        assert_eq!(
            TransferType::Isochronous,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0001))
                .transfer_type()
        );
        assert_eq!(
            TransferType::Bulk,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0010))
                .transfer_type()
        );
        assert_eq!(
            TransferType::Interrupt,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0011))
                .transfer_type()
        );
    }

    #[test]
    fn it_interprets_synchronization_type_in_attributes() {
        assert_eq!(
            IsoSyncType::NoSync,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0001))
                .iso_sync_type()
        );
        assert_eq!(
            IsoSyncType::Asynchronous,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0101))
                .iso_sync_type()
        );
        assert_eq!(
            IsoSyncType::Adaptive,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_1001))
                .iso_sync_type()
        );
        assert_eq!(
            IsoSyncType::Synchronous,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_1101))
                .iso_sync_type()
        );
    }

    #[test]
    fn it_interprets_usage_type_in_attributes() {
        assert_eq!(
            IsoUsageType::Data,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0000_0001))
                .iso_usage_type()
        );
        assert_eq!(
            IsoUsageType::Feedback,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0001_0001))
                .iso_usage_type()
        );
        assert_eq!(
            IsoUsageType::FeedbackData,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0010_0001))
                .iso_usage_type()
        );
        assert_eq!(
            IsoUsageType::Reserved,
            EndpointDescriptor::from(&endpoint_descriptor!(bmAttributes: 0b0011_0001))
                .iso_usage_type()
        );
    }

    #[test]
    fn it_has_max_packet_size() {
        assert_eq!(
            64,
            EndpointDescriptor::from(&endpoint_descriptor!(wMaxPacketSize: 64)).max_packet_size()
        );
        assert_eq!(
            4096,
            EndpointDescriptor::from(&endpoint_descriptor!(wMaxPacketSize: 4096)).max_packet_size()
        );
        assert_eq!(
            65535,
            EndpointDescriptor::from(&endpoint_descriptor!(wMaxPacketSize: 65535))
                .max_packet_size()
        );
    }

    #[test]
    fn it_has_interval() {
        assert_eq!(
            1,
            EndpointDescriptor::from(&endpoint_descriptor!(bInterval: 1)).interval()
        );
        assert_eq!(
            20,
            EndpointDescriptor::from(&endpoint_descriptor!(bInterval: 20)).interval()
        );
        assert_eq!(
            255,
            EndpointDescriptor::from(&endpoint_descriptor!(bInterval: 255)).interval()
        );
    }
}
