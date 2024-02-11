#[cfg(feature = "epd_2in7b")]
use waveshare_epd::epd_2in7b::{pack_buffer, Epd};

#[test]
fn basics() {
    let mut epd = Epd::new().unwrap();
    epd.init().unwrap();
    epd.clear().unwrap();
    epd.sleep().unwrap();
}

#[test]
fn image() {
    let black = image::io::Reader::with_format(
        std::io::Cursor::new(&include_bytes!("wBJwq8ap6D-b.bmp")),
        image::ImageFormat::Bmp,
    )
    .decode()
    .unwrap()
    .into_luma8();
    let black = pack_buffer(&black).unwrap();

    let red = image::io::Reader::with_format(
        std::io::Cursor::new(&include_bytes!("wBJwq8ap6D-r.bmp")),
        image::ImageFormat::Bmp,
    )
    .decode()
    .unwrap()
    .into_luma8();
    let red = pack_buffer(&red).unwrap();

    let mut epd = Epd::new().unwrap();
    epd.init().unwrap();
    epd.display(black.iter().copied(), red.iter().copied())
        .unwrap();
    epd.sleep().unwrap();
}
