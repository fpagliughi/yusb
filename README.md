# yusb

Yet another fork of a fork of a Rust [libusb](https://libusb.info/) wrapper!

Pronounced _yoo-ess-bee_.

This is a fork of Ilya Averyanov's [rusb](https://crates.io/crates/rusb) crate, which itself is a fork of David Cuddeback's [libusb](https://crates.io/crates/libusb) crate.

The initial version of this crate differs from `rusb` in a number of ways:

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

This crate provides a safe wrapper around the native `libusb` library. It applies the RAII pattern
and Rust lifetimes to ensure safe usage of all `libusb` functionality. The RAII pattern ensures that
all acquired resources are released when they're no longer needed, and Rust lifetimes ensure that
resources are released in a proper order.

* [Documentation](https://docs.rs/yusb)

## Dependencies

To use yusb, no extra setup is required as yusb will automatically download the source for libusb and build it.

However if building libusb fails you can also try setting up the native `libusb` library where it can
be found by `pkg-config` or `vcpkg`.

All systems supported by the native `libusb` library are also supported by the `libusb` crate. It's
been tested on Linux, OS X, and Windows.

### Cross-Compiling

The `yusb` crate can be used when cross-compiling to a foreign target. Details on how to
cross-compile `yusb` are explained in the [`libusb1-sys` crate's
README](libusb1-sys/README.md#cross-compiling).

## Usage

Add `yusb` as a dependency in `Cargo.toml`:

```toml
[dependencies]
yusb = "0.1"
```

Import the `yusb` crate. The starting point for nearly all `yusb` functionality is to create a
context object. With a context object, you can list devices, read their descriptors, open them, and
communicate with their endpoints:

```rust
fn main() {
    for device in yusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id());
    }
}
```

## License

Distributed under the [MIT License](LICENSE).

### License note.

If you link native `libusb` (by example using `vendored` features) library statically then
you must follow [GNU LGPL](https://github.com/libusb/libusb/blob/master/COPYING) from libusb.
