//! Nifty 默认规范：gitmoji 前缀 commit subject 与 release 分组。

use std::{collections::HashMap, sync::LazyLock};

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
    "✨", "🎨", "🚀", "🐛", "🚑", "🔥", "💥", "♻️", "🔧", "📝", "👷", "🧹", "⬆️", "🧪", "🔨", "📦", "🎂", "🎉", "🚧", "🚫",
    "💄", "🎀", "⚡", "🔒", "📌",
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

/// 判断码位是否属于 emoji 区段（不含修饰符）。
fn is_emoji_cp(cp: u32) -> bool {
    matches!(cp,
        0x1F000..=0x1FAFF
            | 0x2600..=0x27BF
            | 0x2300..=0x23FF
            | 0x2B00..=0x2BFF
            | 0x2190..=0x21FF
            | 0x203C
            | 0x2049
            | 0x2122
            | 0x2139
            | 0x21A9
            | 0x21AA
            | 0x25AA
            | 0x25AB
            | 0x25B6
            | 0x25C0
            | 0x25FB..=0x25FE
            | 0x2934
            | 0x2935
            | 0x3030
            | 0x303D
            | 0x3297
            | 0x3299
            | 0x00A9
            | 0x00AE)
}

/// 返回 subject 行首的任意 emoji 簇（后跟空格时）。
///
/// 与 [`leading_gitmoji`] 不同，本函数不受已知 gitmoji 列表限制：
/// `🔖`、`🎂` 等 emoji 同样是合法前缀，任何 emoji 前缀都不应被拒绝。
pub fn leading_emoji(subject: &str) -> Option<&str> {
    let mut chars = subject.char_indices().peekable();
    let (_, first) = chars.next()?;
    if !is_emoji_cp(first as u32) {
        return None;
    }
    let mut end = first.len_utf8();
    while let Some((idx, ch)) = chars.next() {
        let cp = ch as u32;
        if cp == 0xFE0F || cp == 0x20E3 {
            end = idx + ch.len_utf8();
        }
        else if cp == 0x200D && chars.peek().is_some_and(|(_, next)| is_emoji_cp(*next as u32)) {
            end = idx + ch.len_utf8();
        }
        else if is_emoji_cp(cp) {
            end = idx + ch.len_utf8();
        }
        else {
            break;
        }
    }
    subject[end..].starts_with(' ').then(|| &subject[..end])
}

/// 去掉 subject 行首 gitmoji 与后续空白。
pub fn strip_gitmoji(subject: &str) -> String {
    if let Some(emoji) = leading_emoji(subject) {
        return subject[emoji.len()..].trim_start().to_string();
    }
    subject.trim().to_string()
}

/// 将 gitmoji 映射到 release 分组。
pub fn section_for_gitmoji(gitmoji: Option<&str>) -> Section {
    gitmoji.and_then(|emoji| GITMOJI_SECTION.get(emoji).copied()).unwrap_or(Section::Other)
}

/// 解析 subject：gitmoji + body + section。
pub fn parse_subject(subject: &str) -> ParsedSubject {
    let gitmoji = leading_emoji(subject).map(str::to_string);
    let section = section_for_gitmoji(gitmoji.as_deref());
    let body = strip_gitmoji(subject);
    ParsedSubject { gitmoji, body, section }
}

pub fn validate_subject(subject: &str) -> bool {
    leading_emoji(subject).is_some()
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
