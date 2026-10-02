//! Nifty 默认规范：gitmoji 前缀 commit subject 与 release 分组。

use std::collections::HashMap;
use std::sync::LazyLock;

/// release 参考稿分组键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Section {
    /// Features (`✨`, `🎨`, `🚀`).
    Features,
    /// Bug fixes (`🐛`, `🚑`, `🔥`).
    Fixes,
    /// Breaking changes (`💥`).
    Breaking,
    /// 其它维护类 gitmoji。
    Other,
}

/// 解析后的 gitmoji subject。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ParsedSubject {
    /// Leading gitmoji if present.
    pub gitmoji: Option<String>,
    /// Subject body without gitmoji prefix.
    pub body: String,
    /// Release section derived from gitmoji.
    pub section: Section,
}

static GITMOJI_SECTION: LazyLock<HashMap<&'static str, Section>> = LazyLock::new(|| {
    HashMap::from([
        ("✨", Section::Features),
        ("🎨", Section::Features),
        ("🚀", Section::Features),
        ("🐛", Section::Fixes),
        ("🚑", Section::Fixes),
        ("🔥", Section::Fixes),
        ("💥", Section::Breaking),
        ("♻️", Section::Other),
        ("🔧", Section::Other),
        ("📝", Section::Other),
        ("👷", Section::Other),
        ("🧹", Section::Other),
        ("⬆️", Section::Other),
        ("🧪", Section::Other),
        ("🔨", Section::Other),
        ("📦", Section::Other),
        ("🎂", Section::Other),
        ("🎉", Section::Other),
        ("🚧", Section::Other),
        ("🚫", Section::Other),
        ("💄", Section::Other),
        ("🎀", Section::Other),
        ("⚡", Section::Other),
        ("🔒", Section::Other),
        ("📌", Section::Other),
    ])
});

static KNOWN_GITMOJIS: [&str; 25] = [
    "✨", "🎨", "🚀", "🐛", "🚑", "🔥", "💥", "♻️", "🔧", "📝", "👷", "🧹", "⬆️", "🧪", "🔨", "📦", "🎂", "🎉",
    "🚧", "🚫", "💄", "🎀", "⚡", "🔒", "📌",
];

/// Nifty 认可的 gitmoji 列表（subject 必须以其中之一开头）。
pub fn known_gitmojis() -> &'static [&'static str] {
    &KNOWN_GITMOJIS
}

/// 返回 subject 行首 gitmoji（若有）。
pub fn leading_gitmoji(subject: &str) -> Option<&'static str> {
    for emoji in KNOWN_GITMOJIS {
        if subject.starts_with(emoji) {
            let rest = subject.strip_prefix(emoji).unwrap_or(subject);
            let rest = rest.strip_prefix('\u{fe0f}').unwrap_or(rest);
            if rest.starts_with(' ') {
                return Some(emoji);
            }
        }
    }
    None
}

/// 去掉 subject 行首 gitmoji 与后续空白。
pub fn strip_gitmoji(subject: &str) -> String {
    if let Some(emoji) = leading_gitmoji(subject) {
        let rest = subject.strip_prefix(emoji).unwrap_or(subject);
        let rest = rest.strip_prefix('\u{fe0f}').unwrap_or(rest);
        return rest.trim_start().to_string();
    }
    subject.trim().to_string()
}

/// 将 gitmoji 映射到 release 分组。
pub fn section_for_gitmoji(gitmoji: Option<&str>) -> Section {
    gitmoji.and_then(|emoji| GITMOJI_SECTION.get(emoji).copied()).unwrap_or(Section::Other)
}

/// 解析 subject：gitmoji + body + section。
pub fn parse_subject(subject: &str) -> ParsedSubject {
    let gitmoji = leading_gitmoji(subject).map(str::to_string);
    let section = section_for_gitmoji(gitmoji.as_deref());
    let body = strip_gitmoji(subject);
    ParsedSubject { gitmoji, body, section }
}

pub fn validate_subject(subject: &str) -> bool {
    leading_gitmoji(subject).is_some()
}

/// 分组标题与空节占位文案。
pub fn section_meta(section: Section) -> (&'static str, &'static str) {
    match section {
        Section::Features => ("## ✨ Features", "(none)"),
        Section::Fixes => ("## 🐛 Bug Fixes", "(none)"),
        Section::Breaking => ("## ⚠️ Breaking Changes", "(none)"),
        Section::Other => ("## 📝 Other", "(none)"),
    }
}

/// 按 release 顺序列出全部分组。
pub fn all_sections() -> [Section; 4] {
    [Section::Features, Section::Fixes, Section::Breaking, Section::Other]
}

/// 分组键的字符串形式（WIT / JSON 边界）。
pub fn section_name(section: Section) -> &'static str {
    match section {
        Section::Features => "features",
        Section::Fixes => "fixes",
        Section::Breaking => "breaking",
        Section::Other => "other",
    }
}

/// 从字符串解析分组键。
pub fn section_from_name(name: &str) -> Option<Section> {
    match name {
        "features" => Some(Section::Features),
        "fixes" => Some(Section::Fixes),
        "breaking" => Some(Section::Breaking),
        "other" => Some(Section::Other),
        _ => None,
    }
}
