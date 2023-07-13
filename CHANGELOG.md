# Change Log for yusb

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

### v0.1.1  2023-07-13

- Cleaned up `Devices`, the iterator for `DeviceList`.
- Added IntoIterator for `DeviceList`. Can now do for loops over the device list itself.
- Updated fields enum values:
    - "C" repr and variants agree with comparable libusb constants.
    - Implemented `From` traits for the ones missing them.
    - `EndpointDescriptor` now uses the `From` traits
- Fixed Windows build warning


### v0.1.0  2023-07-08

Initial fork of [rusb](https://github.com/a1ien/rusb) with modifications from [personal fork](https://github.com/fpagliughi/rusb/tree/cleanup)

This version differs from `rusb` in a number of ways:

- Removes the `UsbContext` trait
    - Consolidates `Context` and `GenericContext` types into a single, concrete `Context` type.
    - Now the generic context is just an instance of `Context` with a _null_ inner pointer.
- The `Device<T>` and `DeviceList<T>` no longer need to be generic over the `Context` type (since there is now only a single context type), and are now just `Device` and `DeviceList`, respectively.
- There is a `Port` type which uniquely identified the physical USB port to which a device in the system is attached.
    - It is a combination of the bus number and ordered list of hub ports
    - This helps to uniquely identify a device when multiple ones are attached with the same VID:PID and no serial number or other distinguishing feature.
    - Individual ports are comparable and can be converted to/from strings that use the Linux _syspath_ format, like **2-1.4.3**.
- The `Speed` type updated:
    - It can be converted to floating-point speed in Mbps, and directly displayed as such.
    - It is ordered and comparable like:
```text
    if (device.speed() < Speed::Super) { println!("Plug the device into a faster port");
```
- Some general cleanup and modernization of the code base.
