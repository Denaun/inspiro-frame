use simplelog::{Config, LevelFilter, SimpleLogger};
use waveshare_epd::epd_2in7b as epd;

#[tokio::main]
pub async fn main() -> anyhow::Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;

    let image = inspiro_bot::generate_image().await?.into_rgb8();
    let image = image::imageops::thumbnail(&image, epd::EPD_HEIGHT as u32, epd::EPD_WIDTH as u32);

    let (black, red) = rgb2bwr::to_bwr_split(image, true);
    let black = epd::pack_buffer(&black).unwrap();
    let red = epd::pack_buffer(&red).unwrap();

    let mut epd = epd::Epd::new()?;
    epd.init()?;
    epd.display(black.iter().copied(), red.iter().copied())?;
    epd.sleep()?;

    Ok(())
}
