use serde_json::{Map, Value};

use crate::theme_support;

pub(crate) fn render_maintainable_theme(
    template: &str,
    theme_name: &str,
    title: &str,
    version: &str,
    app_url: &str,
    asset_version: &str,
) -> String {
    template
        .replace("{{ $title ?? 'Xboard' }}", &escape_html(title))
        .replace("{{ $version ?? '' }}", &escape_html(version))
        .replace("{{ $version ?? 'dev' }}", &escape_html(version))
        .replace("{{ config('app.url') }}", &escape_html(app_url))
        .replace(
            "{{ $asset_version ?? $version ?? 'dev' }}",
            &escape_html(asset_version),
        )
        .replace("/theme/Maintainable/app.css", &format!("/theme/{theme_name}/app.css"))
        .replace("/theme/Maintainable/app.js", &format!("/theme/{theme_name}/app.js"))
        .replace(">Maintainable<", &format!(">{}<", escape_html(theme_name)))
}

pub(crate) fn render_portal_theme(
    template: &str,
    theme_name: &str,
    title: &str,
    version: &str,
    asset_version: &str,
    description: &str,
    logo: &str,
    theme_config: &Map<String, Value>,
) -> String {
    let theme_sidebar = theme_config
        .get("theme_sidebar")
        .and_then(Value::as_str)
        .unwrap_or("light");
    let theme_header = theme_config
        .get("theme_header")
        .and_then(Value::as_str)
        .unwrap_or("dark");
    let theme_color = theme_config
        .get("theme_color")
        .and_then(Value::as_str)
        .unwrap_or("default");
    let background_url = theme_config
        .get("background_url")
        .and_then(Value::as_str)
        .unwrap_or("");
    let custom_html = theme_config
        .get("custom_html")
        .and_then(Value::as_str)
        .unwrap_or("");
    let custom_css = if theme_support::theme_has_asset(theme_name, "assets/custom.css") {
        format!(
            "<link rel=\"stylesheet\" href=\"/theme/{theme_name}/assets/custom.css?v={}\">",
            escape_html(asset_version)
        )
    } else {
        String::new()
    };
    let custom_js = if theme_support::theme_has_asset(theme_name, "assets/custom.js") {
        format!(
            "<script src=\"/theme/{theme_name}/assets/custom.js?v={}\"></script>",
            escape_html(asset_version)
        )
    } else {
        String::new()
    };
    let theme_color_hex = match theme_color {
        "darkblue" => "#3b5998",
        "black" => "#343a40",
        "green" => "#319795",
        _ => "#0665d0",
    };

    template
        .replace("{{$theme}}", &escape_html(theme_name))
        .replace("{{ $theme }}", &escape_html(theme_name))
        .replace("{{ $title }}", &escape_html(title))
        .replace("{{$title}}", &escape_html(title))
        .replace("{{ $version }}", &escape_html(version))
        .replace("{{$version}}", &escape_html(version))
        .replace("{{ $description }}", &escape_html(description))
        .replace("{{$description}}", &escape_html(description))
        .replace("{{ $logo }}", &escape_html(logo))
        .replace("{{$logo}}", &escape_html(logo))
        .replace("{{$theme_config['theme_sidebar']}}", &escape_html(theme_sidebar))
        .replace("{{$theme_config['theme_header']}}", &escape_html(theme_header))
        .replace("{{$theme_config['theme_color']}}", &escape_html(theme_color))
        .replace("{{$theme_config['background_url']}}", &escape_html(background_url))
        .replace("{{$colors[$theme_config['theme_color']]}}", theme_color_hex)
        .replace("{!! $theme_config['custom_html'] !!}", custom_html)
        .replace(
            "@if (file_exists(public_path(\"/theme/{$theme}/assets/custom.css\")))\n        <link rel=\"stylesheet\" href=\"/theme/{{$theme}}/assets/custom.css?v={{$version}}\">\n    @endif",
            &custom_css,
        )
        .replace(
            "@if (file_exists(public_path(\"/theme/{$theme}/assets/custom.js\")))\n    <script src=\"/theme/{{$theme}}/assets/custom.js?v={{$version}}\"></script>\n@endif",
            &custom_js,
        )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#39;")
}
