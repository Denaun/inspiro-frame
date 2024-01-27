const API_URL: &str = "https://inspirobot.me/api?generate=true";

pub fn generate_url() -> anyhow::Result<String> {
    Ok(ureq::get(API_URL).call()?.into_string()?)
}

pub fn generate_image() -> anyhow::Result<image::DynamicImage> {
    let url = generate_url()?;
    fetch_image(&url)
}

pub fn fetch_image(url: &str) -> anyhow::Result<image::DynamicImage> {
    log::info!("Fetching {}", url);
    let response = ureq::get(url).call()?;
    let mut bytes: Vec<u8> = match response.header("Content-Length") {
        Some(len) => Vec::with_capacity(len.parse()?),
        None => Vec::new(),
    };
    response.into_reader().read_to_end(&mut bytes)?;
    let decoder = image::io::Reader::with_format(
        std::io::Cursor::new(bytes),
        image::ImageFormat::from_path(url)?,
    );
    Ok(decoder.decode()?)
}
