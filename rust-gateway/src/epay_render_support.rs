use crate::*;

pub(crate) fn ensure_http_checkout_url(value: &str) -> Result<(), Response<Body>> {
    crate::url_security_support::normalize_http_url(value, false)
        .map(|_| ())
        .map_err(|_| fail_json_response(StatusCode::BAD_REQUEST, "Invalid payment gateway URL"))
}

pub(crate) fn build_epay_auto_post_form(
    submit_url: &str,
    params: &serde_json::Map<String, Value>,
) -> String {
    let inputs = params
        .iter()
        .map(|(key, value)| {
            format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
                escape_html_attr(key),
                escape_html_attr(value.as_str().unwrap_or_default())
            )
        })
        .collect::<Vec<_>>()
        .join("");

    format!(
        "<form id=\"epay_submit\" action=\"{}\" method=\"post\">{}</form><script>document.getElementById('epay_submit').submit();</script>",
        escape_html_attr(submit_url),
        inputs
    )
}

fn escape_html_attr(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
