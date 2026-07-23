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
