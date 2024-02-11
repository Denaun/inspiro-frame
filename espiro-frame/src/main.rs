use anyhow::{anyhow, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{peripherals::Peripherals, task::block_on},
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use log::info;
use waveshare_epd::epd_12in48b::Epd;

#[derive(Debug)]
#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
}
impl Config {
    fn wifi(&self) -> Result<ClientConfiguration> {
        Ok(ClientConfiguration {
            ssid: CONFIG
                .wifi_ssid
                .try_into()
                .map_err(|_| anyhow!("invalid SSID"))?,
            bssid: None,
            auth_method: AuthMethod::WPA2Personal,
            password: CONFIG
                .wifi_password
                .try_into()
                .map_err(|_| anyhow!("invalid password"))?,
            channel: None,
        })
    }
}

fn main() -> Result<()> {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    EspLogger::initialize_default();
    EspLogger {}.set_target_level("waveshare_epd", log::LevelFilter::Debug)?;

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let timer_service = EspTaskTimerService::new()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = AsyncWifi::wrap(
        EspWifi::new(peripherals.modem, sys_loop.clone(), Some(nvs))?,
        sys_loop,
        timer_service,
    )?;
    block_on(connect_wifi(&mut wifi, CONFIG.wifi()?))?;

    let mut epd = Epd::new(peripherals.spi3, peripherals.pins)?;
    block_on(async move {
        epd.init()?;
        epd.clear()?;
        epd.turn_on().await?;
        epd.sleep()?;
        Ok(())
    })
}

async fn connect_wifi(
    wifi: &mut AsyncWifi<EspWifi<'static>>,
    config: ClientConfiguration,
) -> Result<()> {
    let wifi_configuration: Configuration = Configuration::Client(config);
    wifi.set_configuration(&wifi_configuration)?;
    wifi.start().await?;
    info!("Wifi started");
    wifi.connect().await?;
    info!("Wifi connected");
    wifi.wait_netif_up().await?;
    info!("Wifi netif up");
    Ok(())
}
