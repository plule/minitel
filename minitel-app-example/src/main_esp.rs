use log::{debug, error, info};
use minitel::stum::{
    protocol::{Baudrate, RoutingRx, RoutingTx},
    AsyncMinitelReadWrite, AsyncMinitelReadWriteBaudrate,
};
use std::thread::sleep;

use crate::app::App;

pub fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    esp_idf_svc::io::vfs::initialize_eventfd(1).expect("Failed to initialize eventfd");

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    match rt.block_on(async { async_main().await }) {
        Ok(()) => info!("main() finished, reboot."),
        Err(err) => {
            error!("{err:?}");
            // Let them read the error message before rebooting
            sleep(std::time::Duration::from_secs(3));
        }
    }

    esp_idf_svc::hal::reset::restart();
}

async fn async_main() -> std::io::Result<()> {
    // Initialize the minitel
    let mut minitel = minitel::esp_minitel_uart2().unwrap();
    minitel.search_speed().await.unwrap();
    minitel.set_speed(Baudrate::B9600).await.unwrap();
    minitel
        .set_routing(false, RoutingRx::Modem, RoutingTx::Keyboard)
        .await
        .unwrap();

    // Run the app
    App::default().run(&mut minitel).await.unwrap();

    minitel
        .set_routing(true, RoutingRx::Modem, RoutingTx::Keyboard)
        .await
        .unwrap();
    Ok(())
}
