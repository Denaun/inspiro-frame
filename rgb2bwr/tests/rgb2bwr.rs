/// Ensure black and white images are not converted to reddish images.
///
/// If white gets converted to red, when using dithering the error expands to
/// turn black into red as well. This shouldn't occur without dithering unless
/// the image is uniform (although the image will still be black-and-red).
#[test]
fn black_and_white() -> image::ImageResult<()> {
    let image = image::io::Reader::with_format(
        std::io::Cursor::new(include_bytes!("3Ee3DwnMJ0.jpg")),
        image::ImageFormat::Jpeg,
    )
    .decode()
    .unwrap();
    let result = rgb2bwr::to_bwr(image.into_rgb8(), true);
    assert!(result.pixels().filter(|p| p.0 != [255, 0, 0]).count() > result.pixels().len() / 10);
    Ok(())
}
