use oxc_formatter::{JsFormatOptions, QuoteStyle};
use oxc_formatter_core::{IndentStyle, IndentWidth, LineWidth};

/// JavaScript/TypeScript formatter style from `nifty.config` `format.style`.
#[derive(Debug, Clone, Default)]
pub struct FormatStyleOptions {
    pub indent_style: Option<String>,
    pub indent_width: Option<u8>,
    pub line_width: Option<u16>,
    pub quote_style: Option<String>,
}

/// Default style while legacy `oxc_formatter` shim remains: 4 spaces, single quotes, line width 144.
pub fn default_format_options() -> JsFormatOptions {
    JsFormatOptions {
        indent_style: IndentStyle::Space,
        indent_width: IndentWidth::try_from(4).unwrap_or_default(),
        line_width: LineWidth::try_from(144).unwrap_or_default(),
        quote_style: QuoteStyle::Single,
        ..JsFormatOptions::default()
    }
}

pub fn resolve_format_options(style: Option<&FormatStyleOptions>) -> JsFormatOptions {
    let mut options = default_format_options();
    let style = style.cloned().unwrap_or_default();

    if let Some(indent_style) = style.indent_style.as_deref() {
        options.indent_style = match indent_style {
            "tab" => IndentStyle::Tab,
            _ => IndentStyle::Space,
        };
    }
    if let Some(width) = style.indent_width {
        if let Ok(indent_width) = IndentWidth::try_from(width) {
            options.indent_width = indent_width;
        }
    }
    if let Some(width) = style.line_width {
        if let Ok(line_width) = LineWidth::try_from(width) {
            options.line_width = line_width;
        }
    }
    if let Some(quote) = style.quote_style.as_deref() {
        options.quote_style = match quote {
            "double" => QuoteStyle::Double,
            _ => QuoteStyle::Single,
        };
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxc_formatter::QuoteStyle;

    #[test]
    fn resolve_style_from_config_fields() {
        let options = resolve_format_options(Some(&FormatStyleOptions {
            indent_style: Some("space".to_string()),
            indent_width: Some(2),
            line_width: Some(100),
            quote_style: Some("double".to_string()),
        }));
        assert_eq!(options.indent_width.value(), 2);
        assert_eq!(options.line_width.value(), 100);
        assert_eq!(options.quote_style, QuoteStyle::Double);
    }
}
