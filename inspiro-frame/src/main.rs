mod api;

use clap::Parser;
use image::imageops;
use simplelog::{LevelFilter::Info, SimpleLogger};
use waveshare_epd::epd_2in7b as epd;

#[derive(Parser, Debug)]
struct Opt {
    /// Optional URL of the image to display. Will be generated if not specified.
    #[arg()]
    url: Option<String>,
    /// Enable dithering (default)
    #[arg(long, conflicts_with = "no_dither")]
    dither: bool,
    /// Disable dithering
    #[arg(long, conflicts_with = "dither")]
    no_dither: bool,
}

pub fn main() -> anyhow::Result<()> {
    SimpleLogger::init(Info, Default::default())?;

    let Opt {
        url,
        dither,
        no_dither,
    } = Parser::parse();
    let dither = dither || !no_dither;

    let image = match url {
        Some(url) => api::fetch_image(&url),
        None => api::generate_image(),
    }?
    .into_rgb8();
    let image = imageops::thumbnail(&image, epd::EPD_HEIGHT as u32, epd::EPD_WIDTH as u32);

    let (black, red) = rgb2bwr::to_bwr_split(image, dither);
    let black = epd::pack_buffer(&black).unwrap();
    let red = epd::pack_buffer(&red).unwrap();

    let mut epd = epd::Epd::new()?;
    epd.init()?;
    epd.display(black.iter().copied(), red.iter().copied())?;
    epd.sleep()?;

    Ok(())
}
