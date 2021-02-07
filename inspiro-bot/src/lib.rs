mod error;

pub use crate::error::{ApiError as Error, Result};

const API_URL: &str = "https://inspirobot.me/api?generate=true";

pub async fn generate_url() -> Result<String> {
    Ok(reqwest::get(API_URL).await?.text().await?)
}

pub async fn generate_image() -> Result<image::DynamicImage> {
    let url = generate_url().await?;
    let data = reqwest::get(&url).await?.bytes().await?;
    let decoder = image::io::Reader::with_format(
        std::io::Cursor::new(&data),
        image::ImageFormat::from_path(&url)?,
    );
    Ok(decoder.decode()?)
}
