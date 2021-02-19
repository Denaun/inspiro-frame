mod api;
mod config;

use crate::config::Config;
use rppal::gpio::{Gpio, Trigger};
use simple_signal::{self, Signal};
use std::{
    path::PathBuf,
    process::{Command, ExitStatus},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use structopt::StructOpt;
use waveshare_epd::epd_2in7b as epd;

#[derive(StructOpt)]
enum Opt {
    /// Changes the displayed image
    Next {
        /// Optional URL of the image to display. Will be generated if not specified.
        #[structopt()]
        url: Option<String>,
        /// Enable dithering (default)
        #[structopt(long, conflicts_with = "no_dither")]
        dither: bool,
        /// Disable dithering
        #[structopt(long, conflicts_with = "dither")]
        no_dither: bool,
    },
    /// Listen for interrupts to run the configured commands
    Listen {
        #[structopt(parse(from_os_str))]
        config: PathBuf,
    },
}

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default())?;

    let opt = Opt::from_args();
    match opt {
        Opt::Next {
            url,
            dither,
            no_dither,
        } => next_image(url.as_deref(), dither || !no_dither).await?,
        Opt::Listen { config } => {
            let config = config::load(config)?;
            listen(config)?
        }
    }

    Ok(())
}

async fn next_image(url: Option<&str>, dither: bool) -> anyhow::Result<()> {
    let image = match url {
        Some(url) => api::fetch_image(url).await,
        None => api::generate_image().await,
    }?
    .into_rgb8();
    let image = image::imageops::thumbnail(&image, epd::EPD_HEIGHT as u32, epd::EPD_WIDTH as u32);

    let (black, red) = rgb2bwr::to_bwr_split(image, dither);
    let black = epd::pack_buffer(&black).unwrap();
    let red = epd::pack_buffer(&red).unwrap();

    let mut epd = epd::Epd::new()?;
    epd.init()?;
    epd.display(black.iter().copied(), red.iter().copied())?;
    epd.sleep()?;

    Ok(())
}

fn listen(config: Config) -> anyhow::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    simple_signal::set_handler(&[Signal::Int, Signal::Term], move |_signals| {
        r.store(false, Ordering::SeqCst);
    });

    log::debug!("Configuring interrupts.");
    let gpio = Gpio::new()?;
    let pins = config
        .gpio
        .into_iter()
        .map(|(pin, command)| -> anyhow::Result<_> {
            log::debug!("Listening on {}", pin);
            let mut key = gpio.get(pin)?.into_input();
            key.set_async_interrupt(Trigger::RisingEdge, move |_| {
                log::info!("Launching {:?}", command);
                if let Err(e) = run_sync(&command) {
                    log::error!("Failed to handle key: {}", e);
                }
            })?;
            Ok(key)
        })
        .collect::<Result<Vec<_>, _>>()?;

    log::info!("Ready.");
    while running.load(Ordering::SeqCst) {}
    log::info!("Exiting...");
    // Interrupts are lost when the pins are dropped: ensure this happens here
    // and not earlier due to optimizations.
    drop(pins);
    Ok(())
}

fn run_sync(command: &[String]) -> anyhow::Result<ExitStatus> {
    let exit_status = Command::new(&command[0])
        .args(&command[1..])
        .spawn()?
        .wait()?;
    Ok(exit_status)
}
