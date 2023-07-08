use yusb as usb;

fn main() {
    let version = usb::version();

    println!(
        "libusb v{}.{}.{}.{}{}",
        version.major(),
        version.minor(),
        version.micro(),
        version.nano(),
        version.rc().unwrap_or("")
    );

    let mut context = match usb::Context::new() {
        Ok(c) => c,
        Err(e) => panic!("libusb::Context::new(): {}", e),
    };

    context.set_log_level(usb::LogLevel::Debug);
    context.set_log_level(usb::LogLevel::Info);
    context.set_log_level(usb::LogLevel::Warning);
    context.set_log_level(usb::LogLevel::Error);
    context.set_log_level(usb::LogLevel::None);

    println!("has capability? {}", usb::has_capability());
    println!("has hotplug? {}", usb::has_hotplug());
    println!("has HID access? {}", usb::has_hid_access());
    println!(
        "supports detach kernel driver? {}",
        usb::supports_detach_kernel_driver()
    )
}
