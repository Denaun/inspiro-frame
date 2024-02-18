use anyhow::{anyhow, bail, ensure, Result};
use core::time::Duration;
use embedded_svc::http::client::Client as HttpClient;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{peripherals::Peripherals, task::block_on},
    http::{
        client::{Configuration as HttpConfiguration, EspHttpConnection},
        headers::{content_len, content_type, ContentLenParseBuf},
    },
    io::Read,
    log::EspLogger,
    nvs::EspDefaultNvsPartition,
    sys::{
        esp, esp_deep_sleep_start, esp_sleep_disable_wakeup_source, esp_sleep_enable_timer_wakeup,
        esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL,
    },
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, AuthMethod, ClientConfiguration, Configuration, EspWifi},
};
use log::{error, info};
use waveshare_epd::epd_12in48b::{
    Epd, EPD_HEIGHT, EPD_WIDTH, HALF_HEIGHT, LEFT_WIDTH, RIGHT_WIDTH,
};

#[derive(Debug)]
#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
    #[default("")]
    dashboard_url: &'static str,
    #[default("")]
    mate_endpoint: &'static str,
    #[default(6)]
    refreshes_per_day: u64,
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

    let mut http_client = HttpClient::wrap(EspHttpConnection::new(&HttpConfiguration {
        timeout: Some(Duration::from_secs(30)),
        ..Default::default()
    })?);
    let id = take_screenshot(&mut http_client, CONFIG.mate_endpoint, CONFIG.dashboard_url)?;
    info!("Created screenshot '{}'", id);

    let mut epd = Epd::new(peripherals.spi3, peripherals.pins)?;
    epd.init()?;
    match display(&mut http_client, CONFIG.mate_endpoint, &mut epd, &id) {
        Err(e) => {
            error!("Display failed: {}", e);
            epd.clear()?;
        }
        _ => {}
    }
    block_on(epd.turn_on())?;
    epd.sleep()?;

    // TODO: esp-rs/esp-idf-hal#287 - Use sleep API.
    unsafe {
        esp!(esp_sleep_disable_wakeup_source(
            esp_sleep_source_t_ESP_SLEEP_WAKEUP_ALL
        ))?;
        esp!(esp_sleep_enable_timer_wakeup(
            86_400_000_000u64 / CONFIG.refreshes_per_day
        ))?;
        esp_deep_sleep_start()
    }
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

fn take_screenshot(
    client: &mut HttpClient<EspHttpConnection>,
    endpoint: &str,
    dashboard: &str,
) -> Result<String> {
    let uri = format!("{}/screenshots", endpoint);
    info!("Taking screenshot of {} on {}", dashboard, uri);
    let data = format!("{{\"url\":\"{dashboard}\",\"width\":{EPD_WIDTH},\"height\":{EPD_HEIGHT}}}")
        .into_bytes();
    let mut len_buf = ContentLenParseBuf::new();
    let headers = [
        content_type("application/json"),
        content_len(data.len() as u64, &mut len_buf),
    ];
    let mut request = client.post(&uri, &headers)?;
    request.write(&data)?;
    let mut response = request.submit()?;
    match response.status() {
        200..=299 => {}
        status => bail!("Unexpected response code: {}", status),
    }
    let len = response
        .header("Content-Length")
        .ok_or(anyhow!("endpoint didn't set Content-Length"))?
        .parse()?;
    let mut buf = vec![0; len];
    response.read_exact(&mut buf)?;
    Ok(String::from_utf8(buf)?)
}

fn display(
    client: &mut HttpClient<EspHttpConnection>,
    endpoint: &str,
    epd: &mut Epd,
    id: &str,
) -> Result<()> {
    info!("s2");
    let (white, red) = fetch_quadrant(client, endpoint, &id, 0, 0, LEFT_WIDTH, HALF_HEIGHT)?;
    epd.s2_display(&white.try_into().unwrap(), &red.try_into().unwrap())?;
    info!("m2");
    let (white, red) = fetch_quadrant(
        client,
        endpoint,
        &id,
        LEFT_WIDTH,
        0,
        RIGHT_WIDTH,
        HALF_HEIGHT,
    )?;
    epd.m2_display(&white.try_into().unwrap(), &red.try_into().unwrap())?;
    info!("m1");
    let (white, red) = fetch_quadrant(
        client,
        endpoint,
        &id,
        0,
        HALF_HEIGHT,
        LEFT_WIDTH,
        HALF_HEIGHT,
    )?;
    epd.m1_display(&white.try_into().unwrap(), &red.try_into().unwrap())?;
    info!("s1");
    let (white, red) = fetch_quadrant(
        client,
        endpoint,
        &id,
        LEFT_WIDTH,
        HALF_HEIGHT,
        RIGHT_WIDTH,
        HALF_HEIGHT,
    )?;
    epd.s1_display(&white.try_into().unwrap(), &red.try_into().unwrap())?;
    Ok(())
}

fn fetch_quadrant(
    client: &mut HttpClient<EspHttpConnection>,
    endpoint: &str,
    id: &str,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> Result<(Vec<u8>, Vec<u8>)> {
    let uri = format!(
        "{endpoint}/screenshots/{id}?x={x}&y={y}&width={width}&height={height}&format=bwr-raw"
    );
    info!("Fetching quadrant from {}", uri);
    let mut response = client.get(&uri)?.submit()?;
    match response.status() {
        200..=299 => {}
        status => bail!("Unexpected response code: {}", status),
    }
    let len: usize = response
        .header("Content-Length")
        .ok_or(anyhow!("endpoint didn't set Content-Length"))?
        .parse()?;
    let expected = 2 * width * height / 8;
    ensure!(len == expected, "expected {expected} bytes, got {len}");
    let mut white = vec![0; len / 2];
    let mut red = vec![0; len / 2];
    response.read_exact(&mut white)?;
    response.read_exact(&mut red)?;
    Ok((white, red))
}
