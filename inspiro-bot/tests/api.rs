use tokio_test::*;

#[test]
fn url() {
    const PREFIX: &str = "https://generated.inspirobot.me/a/";
    let url = assert_ok!(block_on(inspiro_bot::generate_url()));
    assert_eq!(&url[..PREFIX.len()], PREFIX)
}

#[test]
fn image() {
    let _ = assert_ok!(block_on(inspiro_bot::generate_image()));
}
