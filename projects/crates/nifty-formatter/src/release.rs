//! Release note section assembly.

use std::collections::HashMap;

use nifty_types::{all_sections, section_meta, GithubAuthor, Section};

use crate::render::render_section_bullets;

/// One commit entry for release formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReleaseCommit<'a> {
    /// Subject body without gitmoji prefix.
    pub body: &'a str,
    /// Author email.
    pub email: &'a str,
    /// Author display name.
    pub author: &'a str,
    /// Release section.
    pub section: Section,
}

/// Render a single release section (`## ✨ Features` + bullets).
pub fn format_release_section(
    section: Section,
    commits: &[(String, String, String)],
    map: &HashMap<String, GithubAuthor>,
) -> String {
    let (heading, _) = section_meta(section);
    let bullets = render_section_bullets(section, commits, map);
    format!("{heading}\n\n{bullets}")
}

/// Render a full release note grouped by gitmoji section.
pub fn format_release_notes(commits: &[ReleaseCommit<'_>], map: &HashMap<String, GithubAuthor>) -> String {
    all_sections()
        .into_iter()
        .map(|section| {
            let grouped: Vec<(String, String, String)> = commits
                .iter()
                .filter(|commit| commit.section == section)
                .map(|commit| {
                    (
                        commit.body.to_string(),
                        commit.email.to_string(),
                        commit.author.to_string(),
                    )
                })
                .collect();
            format_release_section(section, &grouped, map)
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nifty_types::Section;

    use super::{format_release_notes, ReleaseCommit};

    #[test]
    fn formats_grouped_release_notes() {
        let commits = [ReleaseCommit {
            body: "Add feature",
            email: "dev@example.com",
            author: "Dev",
            section: Section::Features,
        }];
        let notes = format_release_notes(&commits, &HashMap::new());
        assert!(notes.contains("## ✨ Features"));
        assert!(notes.contains("- Add feature (@Dev)"));
    }
}
