use esp_idf_svc::{
    hal::{peripherals::Peripherals, task::block_on},
    log::EspLogger,
};
use waveshare_epd::epd_12in48b::Epd;

fn main() -> anyhow::Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();
    EspLogger {}.set_target_level("waveshare_epd", log::LevelFilter::Debug)?;

    let peripherals = Peripherals::take()?;
    let mut epd = Epd::new(peripherals.spi3, peripherals.pins)?;
    block_on(async move {
        epd.init()?;
        epd.clear()?;
        epd.turn_on().await?;
        epd.sleep()?;
        Ok(())
    })
}
