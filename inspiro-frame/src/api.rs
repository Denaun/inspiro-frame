const API_URL: &str = "https://inspirobot.me/api?generate=true";

pub async fn generate_url() -> anyhow::Result<String> {
    Ok(reqwest::get(API_URL).await?.text().await?)
}

pub async fn generate_image() -> anyhow::Result<image::DynamicImage> {
    let url = generate_url().await?;
    fetch_image(&url).await
}

pub async fn fetch_image(url: &str) -> anyhow::Result<image::DynamicImage> {
    log::info!("Fetching {}", url);
    let data = reqwest::get(url).await?.bytes().await?;
    let decoder = image::io::Reader::with_format(
        std::io::Cursor::new(&data),
        image::ImageFormat::from_path(url)?,
    );
    Ok(decoder.decode()?)
}
