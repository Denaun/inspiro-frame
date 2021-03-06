use tokio_test::*;

#[test]
fn url() {
    const PREFIX: &str = "https://generated.inspirobot.me/a/";
    let url = assert_ok!(block_on(inspiro_frame::api::generate_url()));
    assert_eq!(&url[..PREFIX.len()], PREFIX)
}

#[test]
fn image() {
    let image = assert_ok!(block_on(inspiro_frame::api::fetch_image(
        "https://generated.inspirobot.me/a/3Ee3DwnMJ0.jpg"
    )));
    assert_eq!(
        image,
        image::io::Reader::with_format(
            std::io::Cursor::new(include_bytes!("3Ee3DwnMJ0.jpg")),
            image::ImageFormat::Jpeg,
        )
        .decode()
        .unwrap()
    );
}
