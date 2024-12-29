fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("Hello, world!");

    unsafe {
        let stack = esp_idf_svc::sys::uxTaskGetStackHighWaterMark(std::ptr::null_mut());
        println!("Stack high water mark: {}", stack);
    }

    let mut minitel = minitel::esp_minitel_uart2().unwrap();

    minitel.clear_screen().unwrap();
    minitel.set_pos(10, 10).unwrap();
    minitel.write_str("Hello, world!").unwrap();

    loop {
        let byte = minitel.read_byte().unwrap();
        minitel.write_byte(byte).unwrap();
    }
}
