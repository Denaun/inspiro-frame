#[toml_cfg::toml_config]
struct Config {
    #[default("")]
    wifi_ssid: &'static str,
}

fn main() {
    if CONFIG.wifi_ssid == "" {
        panic!("Missing or invalid config");
    }

    embuild::espidf::sysenv::output();
}
