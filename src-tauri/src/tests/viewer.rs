use super::*;

#[test]
fn parse_valid_uri() {
    assert_eq!(
        parse_view_uri("sharaku://localhost/view/42/0"),
        Some((42, 0))
    );
}

#[test]
fn parse_uri_without_localhost() {
    assert_eq!(parse_view_uri("sharaku://view/1/3"), Some((1, 3)));
}

#[test]
fn parse_uri_with_query() {
    assert_eq!(
        parse_view_uri("sharaku://localhost/view/5/0?t=123"),
        Some((5, 0))
    );
}

#[test]
fn parse_invalid_uri_no_view() {
    assert_eq!(parse_view_uri("sharaku://localhost/other/1/0"), None);
}

#[test]
fn parse_invalid_uri_missing_page() {
    assert_eq!(parse_view_uri("sharaku://localhost/view/1"), None);
}

#[test]
fn parse_invalid_uri_non_numeric() {
    assert_eq!(parse_view_uri("sharaku://localhost/view/abc/0"), None);
}

#[test]
fn content_type_jpeg() {
    assert_eq!(content_type_from_path("/path/to/image.jpg"), "image/jpeg");
    assert_eq!(content_type_from_path("/path/to/image.JPEG"), "image/jpeg");
}

#[test]
fn content_type_png() {
    assert_eq!(content_type_from_path("/path/to/image.png"), "image/png");
}

#[test]
fn content_type_gif() {
    assert_eq!(content_type_from_path("/path/to/image.gif"), "image/gif");
}

#[test]
fn content_type_webp() {
    assert_eq!(content_type_from_path("/path/to/image.webp"), "image/webp");
}

#[test]
fn content_type_bmp() {
    assert_eq!(content_type_from_path("/path/to/image.bmp"), "image/bmp");
}

#[test]
fn content_type_unknown() {
    assert_eq!(
        content_type_from_path("/path/to/file.xyz"),
        "application/octet-stream"
    );
}
