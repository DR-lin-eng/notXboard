pub(crate) fn sanitize_rich_html(value: &str) -> String {
    ammonia::Builder::default().clean(value).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rich_html_keeps_basic_formatting_but_removes_active_content() {
        let cleaned = sanitize_rich_html(
            r#"<p>Hello <strong>world</strong></p><img src=x onerror=alert(1)><script>alert(2)</script>"#,
        );
        assert!(cleaned.contains("<strong>world</strong>"));
        assert!(!cleaned.contains("onerror"));
        assert!(!cleaned.contains("<script"));
        assert!(!cleaned.contains("alert(2)"));
    }
}
