// yusb/src/fields.rs
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

use crate::Error;
use libc::c_int;
use libusb1_sys::constants::*;
use std::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Device speeds.
///
/// Indicates the negotiated speed for the device.
///
/// Note that the enum variants are ordered by increasing speed, and
/// are comparable like:
/// ```text
/// if (dev.speed() < Speed::Super) {
///     println!("Plug the device into a faster port");
/// }
/// ```
///
/// - [libusb_supported_speed](http://libusb.sourceforge.net/api-1.0/group__libusb__dev.html#ga1454797ecc0de4d084c1619c420014f6)
/// - [USB release versions](https://en.wikipedia.org/wiki/USB#Release_versions)
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[non_exhaustive]
pub enum Speed {
    /// The operating system doesn't know the device speed.
    Unknown,
    /// The device is operating at low speed (1.5 Mbps).
    Low,
    /// The device is operating at full speed (12 Mbps).
    Full,
    /// The device is operating at high speed (480 Mbps).
    High,
    /// The device is operating at super speed (5 Gbps).
    Super,
    /// The device is operating at super speed (10 Gbps).
    SuperPlus,
}

impl Speed {
    /// Gets the speed in floating point megabits per second.
    /// If the speed is unknown, it is reported as 0.0
    pub fn as_mbps(&self) -> f64 {
        use Speed::*;
        match *self {
            Low => 1.5,
            Full => 12.0,
            High => 480.0,
            Super => 5000.0,
            SuperPlus => 10000.0,
            _ => 0.0,
        }
    }
}

impl fmt::Display for Speed {
    /// Displays the speed in Mbps
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Speed::Unknown => write!(f, "Unknown"),
            _ => write!(f, "{} Mbps", self.as_mbps()),
        }
    }
}

impl From<c_int> for Speed {
    fn from(n: c_int) -> Self {
        use Speed::*;
        match n {
            LIBUSB_SPEED_LOW => Low,
            LIBUSB_SPEED_FULL => Full,
            LIBUSB_SPEED_HIGH => High,
            LIBUSB_SPEED_SUPER => Super,
            LIBUSB_SPEED_SUPER_PLUS => SuperPlus,
            LIBUSB_SPEED_UNKNOWN | _ => Unknown,
        }
    }
}

/// Transfer and endpoint directions.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Direction {
    /// Direction for read (device to host) transfers.
    In,
    /// Direction for write (host to device) transfers.
    Out,
}

/// An endpoint's transfer type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TransferType {
    /// Control endpoint.
    Control,
    /// Isochronous endpoint.
    Isochronous,
    /// Bulk endpoint.
    Bulk,
    /// Interrupt endpoint.
    Interrupt,
}

/// Isochronous synchronization mode.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SyncType {
    /// No synchronisation.
    NoSync,
    /// Asynchronous.
    Asynchronous,
    /// Adaptive.
    Adaptive,
    /// Synchronous.
    Synchronous,
}

/// Isochronous usage type.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum UsageType {
    /// Data endpoint.
    Data,
    /// Feedback endpoint.
    Feedback,
    /// Explicit feedback data endpoint.
    FeedbackData,
    /// Reserved.
    Reserved,
}

/// Types of control transfers.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum RequestType {
    /// Requests that are defined by the USB standard.
    Standard,
    /// Requests that are defined by a device class, e.g., HID.
    Class,
    /// Vendor-specific requests.
    Vendor,
    /// Reserved for future use.
    Reserved,
}

/// Recipients of control transfers.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Recipient {
    /// The recipient is a device.
    Device,
    /// The recipient is an interface.
    Interface,
    /// The recipient is an endpoint.
    Endpoint,
    /// Other.
    Other,
}

/// The unique port for a USB device.
///
/// This is the combination of the bus number and all the hub ports through
/// which a device is connected.
#[derive(Debug, Clone)]
pub struct Port {
    // The USB bus
    bus: u8,
    // The (ordered) hub ports through which the device is connected.
    ports: Vec<u8>,
}

impl Port {
    /// Create a USB port structure from its components.
    pub fn new(bus: u8, ports: Vec<u8>) -> Self {
        Self { bus, ports }
    }
}

impl fmt::Display for Port {
    /// Outputs a string describing the unique port to which the device is
    /// attached.
    ///
    /// The port string is in the form:
    /// ```text
    /// <bus>[-<port>[.<port>[.<port>[...]]]]
    /// ```
    ///
    /// like `4-1.2`. This can be used on any system, but is especially useful
    /// in Linux as it is compatible with the port designation in the device
    /// path and system path for USB devices.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.bus)?;

        if !self.ports.is_empty() {
            write!(f, "-{}", self.ports[0])?;

            for port in self.ports.iter().skip(1) {
                write!(f, ".{}", port)?;
            }
        }
        Ok(())
    }
}

impl FromStr for Port {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v: Vec<&str> = s.split('-').collect();
        if v.is_empty() || v[0].is_empty() {
            return Err(Error::NotFound);
        }

        let bus: u8 = v[0].parse().map_err(|_| Error::NotFound)?;

        let ports: Vec<u8> = if v.len() < 2 {
            vec![]
        } else {
            v[1].split('.')
                .map(|s| s.parse::<u8>().map_err(|_| Error::NotFound))
                .collect::<Result<Vec<_>, _>>()?
        };

        Ok(Port { bus, ports })
    }
}

impl PartialEq for Port {
    fn eq(&self, other: &Self) -> bool {
        self.bus == other.bus && self.ports == other.ports
    }
}

impl Eq for Port {}

/// A three-part version consisting of major, minor, and sub minor components.
///
/// This can be used to represent versions of the format `J.M.N`, where `J` is the major version,
/// `M` is the minor version, and `N` is the sub minor version. A version is constructed by
/// providing the fields in the same order to the tuple. For example:
///
/// ```
/// yusb::Version(0, 2, 1);
/// ```
///
/// represents the version 0.2.1.
///
/// The intended use case of `Version` is to extract meaning from the version fields in USB
/// descriptors, such as `bcdUSB` and `bcdDevice` in device descriptors.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version(pub u8, pub u8, pub u8);

impl Version {
    /// Extracts a version from a binary coded decimal (BCD) field. BCD fields exist in USB
    /// descriptors as 16-bit integers encoding a version as `0xJJMN`, where `JJ` is the major
    /// version, `M` is the minor version, and `N` is the sub minor version. For example, 2.0 is
    /// encoded as `0x0200` and 1.1 is encoded as `0x0110`.
    pub fn from_bcd(raw: u16) -> Self {
        let sub = (raw & 0x000F) as u8;
        let minor = ((raw >> 4) & 0x000F) as u8;
        let major = (((raw >> 8) & 0x000F) + (10 * ((raw >> 12) & 0x000F))) as u8;
        Version(major, minor, sub)
    }

    /// Returns the major version.
    pub fn major(&self) -> u8 {
        self.0
    }

    /// Returns the minor version.
    pub fn minor(&self) -> u8 {
        self.1
    }

    /// Returns the sub minor version.
    pub fn sub_minor(&self) -> u8 {
        self.2
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

/// Builds a value for the `bmRequestType` field of a control transfer setup packet.
///
/// The `bmRequestType` field of a USB control transfer setup packet is a bit field specifying
/// three parameters, which are given to this function by corresponding enum values.
///
/// ## Examples
///
/// The following example returns a `bmRequestType` value for a standard inbound transfer from the
/// device, which could be used for reading a device's descriptors:
///
/// ```no_run
/// use yusb::{Direction,RequestType,Recipient};
///
/// yusb::request_type(Direction::In, RequestType::Standard, Recipient::Device);
/// ```
pub const fn request_type(
    direction: Direction,
    request_type: RequestType,
    recipient: Recipient,
) -> u8 {
    let mut value: u8 = match direction {
        Direction::Out => LIBUSB_ENDPOINT_OUT,
        Direction::In => LIBUSB_ENDPOINT_IN,
    };

    value |= match request_type {
        RequestType::Standard => LIBUSB_REQUEST_TYPE_STANDARD,
        RequestType::Class => LIBUSB_REQUEST_TYPE_CLASS,
        RequestType::Vendor => LIBUSB_REQUEST_TYPE_VENDOR,
        RequestType::Reserved => LIBUSB_REQUEST_TYPE_RESERVED,
    };

    value |= match recipient {
        Recipient::Device => LIBUSB_RECIPIENT_DEVICE,
        Recipient::Interface => LIBUSB_RECIPIENT_INTERFACE,
        Recipient::Endpoint => LIBUSB_RECIPIENT_ENDPOINT,
        Recipient::Other => LIBUSB_RECIPIENT_OTHER,
    };

    value
}

/////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::*;

    // Port

    #[test]
    fn port_compares_properly() {
        let port = Port::new(1, vec![2u8, 3u8]);

        assert_eq!(port, Port::new(1, vec![2u8, 3u8]));
        assert_ne!(port, Port::new(1, vec![2u8, 9u8]));
        assert_ne!(port, Port::new(1, vec![2u8, 3u8, 9u8]));
    }

    #[test]
    fn port_to_string() {
        let port = Port::new(1, vec![2u8, 3u8]);
        assert_eq!("1-2.3", port.to_string().as_str());

        let port = Port::new(3, vec![]);
        assert_eq!("3", port.to_string().as_str());
    }

    #[test]
    fn port_from_string() {
        let port = Port::from_str("1-2.3.4").unwrap();
        assert_eq!(Port::new(1, vec![2u8, 3u8, 4u8]), port);

        let port = Port::from_str("1-2").unwrap();
        assert_eq!(Port::new(1, vec![2u8]), port);

        let port = Port::from_str("1").unwrap();
        assert_eq!(Port::new(1, vec![]), port);

        let err = Port::from_str("bubba").unwrap_err();
        assert_eq!(Error::NotFound, err);

        let err = Port::from_str("1-").unwrap_err();
        assert_eq!(Error::NotFound, err);
    }

    // Version

    #[test]
    fn version_returns_major_version() {
        assert_eq!(1, Version(1, 0, 0).major());
        assert_eq!(2, Version(2, 0, 0).major());
    }

    #[test]
    fn version_returns_minor_version() {
        assert_eq!(1, Version(0, 1, 0).minor());
        assert_eq!(2, Version(0, 2, 0).minor());
    }

    #[test]
    fn version_returns_sub_minor_version() {
        assert_eq!(1, Version(0, 0, 1).sub_minor());
        assert_eq!(2, Version(0, 0, 2).sub_minor());
    }

    #[test]
    fn version_parses_major_version() {
        assert_eq!(3, Version::from_bcd(0x0300).major());
    }

    #[test]
    fn version_parses_long_major_version() {
        assert_eq!(12, Version::from_bcd(0x1200).major());
    }

    #[test]
    fn version_parses_minor_version() {
        assert_eq!(1, Version::from_bcd(0x0010).minor());
        assert_eq!(2, Version::from_bcd(0x0020).minor());
    }

    #[test]
    fn version_parses_sub_minor_version() {
        assert_eq!(1, Version::from_bcd(0x0001).sub_minor());
        assert_eq!(2, Version::from_bcd(0x0002).sub_minor());
    }

    #[test]
    fn version_parses_full_version() {
        assert_eq!(Version(12, 3, 4), Version::from_bcd(0x1234));
    }

    #[test]
    fn version_display() {
        assert_eq!(Version(2, 45, 13).to_string(), "2.45.13");
    }

    #[test]
    fn version_ord() {
        assert!(Version(0, 0, 0) < Version(1, 2, 3));
        assert!(Version(1, 0, 0) < Version(1, 2, 3));
        assert!(Version(1, 2, 0) < Version(1, 2, 3));
        assert!(Version(1, 2, 0) < Version(1, 3, 0));
        assert!(Version(255, 255, 255) > Version(254, 0, 0));
        assert!(Version(0, 255, 0) > Version(0, 254, 255));
    }

    // request_type for direction

    #[test]
    fn request_type_builds_value_for_out_direction() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x80,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_in_direction() {
        assert_eq!(
            request_type(Direction::In, RequestType::Standard, Recipient::Device) & 0x80,
            0x80
        );
    }

    // request_type for request type

    #[test]
    fn request_type_builds_value_for_standard_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x60,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_class_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Class, Recipient::Device) & 0x60,
            0x20
        );
    }

    #[test]
    fn request_type_builds_value_for_vendor_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Vendor, Recipient::Device) & 0x60,
            0x40
        );
    }

    #[test]
    fn request_type_builds_value_for_reserved_request() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Reserved, Recipient::Device) & 0x60,
            0x60
        );
    }

    // request_type for recipient

    #[test]
    fn request_type_builds_value_for_device_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Device) & 0x0F,
            0x00
        );
    }

    #[test]
    fn request_type_builds_value_for_interface_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Interface) & 0x0F,
            0x01
        );
    }

    #[test]
    fn request_type_builds_value_for_endpoint_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Endpoint) & 0x0F,
            0x02
        );
    }

    #[test]
    fn request_type_builds_value_for_other_recipient() {
        assert_eq!(
            request_type(Direction::Out, RequestType::Standard, Recipient::Other) & 0x0F,
            0x03
        );
    }
}
