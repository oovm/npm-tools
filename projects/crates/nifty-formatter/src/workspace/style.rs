use oak_typescript::formatter::FormatOptions;

/// JavaScript/TypeScript formatter style from the workspace format configuration.
#[derive(Debug, Clone, Default)]
pub struct FormatStyleOptions {
    pub indent_style: Option<String>,
    pub indent_width: Option<u8>,
    pub line_width: Option<u16>,
    pub quote_style: Option<String>,
}

/// Default Oak formatter options.
pub fn default_format_options() -> FormatOptions {
    FormatOptions { indent_width: 4, line_width: 144, type_erasure: false }
}

/// Resolve workspace style into the Oak formatter contract.
pub fn resolve_format_options(style: Option<&FormatStyleOptions>) -> Result<FormatOptions, String> {
    let style = style.cloned().unwrap_or_default();
    if matches!(style.indent_style.as_deref(), Some("tab")) {
        return Err("format.style.indentStyle=tab is not supported by Oak format".to_string());
    }
    if matches!(style.quote_style.as_deref(), Some("double")) {
        return Err("format.style.quoteStyle=double is not supported by Oak format".to_string());
    }
    Ok(FormatOptions {
        indent_width: style.indent_width.unwrap_or(4),
        line_width: usize::from(style.line_width.unwrap_or(144)),
        type_erasure: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_style_from_config_fields() {
        let options = resolve_format_options(Some(&FormatStyleOptions {
            indent_style: Some("space".to_string()),
            indent_width: Some(2),
            line_width: Some(100),
            quote_style: Some("single".to_string()),
        }))
        .expect("supported style");
        assert_eq!(options.indent_width, 2);
        assert_eq!(options.line_width, 100);
    }

    #[test]
    fn rejects_styles_not_represented_by_oak() {
        let style = FormatStyleOptions {
            quote_style: Some("double".to_string()),
            ..FormatStyleOptions::default()
        };
        assert!(resolve_format_options(Some(&style)).is_err());
    }
}
