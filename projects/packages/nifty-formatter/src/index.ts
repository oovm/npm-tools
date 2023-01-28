export type ReleaseSection = "features" | "fixes" | "breaking" | "other";

export type GithubAuthor = {
    id?: number;
    login?: string;
};

const SECTION_HEADINGS: Record<ReleaseSection, string> = {
    features: "## ✨ Features",
    fixes: "## 🐛 Bug Fixes",
    breaking: "## ⚠️ Breaking Changes",
    other: "## 📝 Other",
};

/** Format a commit subject as `<gitmoji> <body>`. */
export function formatSubject(gitmoji: string, body: string): string {
    const trimmed = body.trim();
    return trimmed ? `${gitmoji} ${trimmed}` : gitmoji;
}

/** Markdown author mention for release bullets. */
export function authorMention(
    email: string,
    authorName: string,
    authorMap: Record<string, GithubAuthor> = {},
): string {
    const mapped = authorMap[email.trim().toLowerCase()];
    if (mapped?.login) {
        return `@${mapped.login}`;
    }
    const name = authorName.trim();
    if (name) {
        return `@${name.replace(/\s+/g, "")}`;
    }
    const local = email.split("@")[0]?.trim();
    return local ? `@${local}` : "@unknown";
}

/** Single release bullet: `- body (@user)`. */
export function commitBullet(
    body: string,
    email: string,
    authorName: string,
    authorMap: Record<string, GithubAuthor> = {},
): string {
    return `- ${body} (${authorMention(email, authorName, authorMap)})`;
}

/** Render bullets for one release section. */
export function formatSectionBullets(
    commits: Array<{ body: string; email: string; author: string }>,
    authorMap: Record<string, GithubAuthor> = {},
): string {
    if (commits.length === 0) {
        return "(none)";
    }
    return commits.map((commit) => commitBullet(commit.body, commit.email, commit.author, authorMap)).join("\n");
}

/** Render a release section heading plus bullets. */
export function formatReleaseSection(
    section: ReleaseSection,
    commits: Array<{ body: string; email: string; author: string }>,
    authorMap: Record<string, GithubAuthor> = {},
): string {
    const heading = SECTION_HEADINGS[section];
    const bullets = formatSectionBullets(commits, authorMap);
    return `${heading}\n\n${bullets}`;
}

/** Render a full release note grouped by section. */
export function formatReleaseNotes(
    commits: Array<{ body: string; email: string; author: string; section: ReleaseSection }>,
    authorMap: Record<string, GithubAuthor> = {},
): string {
    const sections: ReleaseSection[] = ["features", "fixes", "breaking", "other"];
    return sections
        .map((section) =>
            formatReleaseSection(
                section,
                commits.filter((commit) => commit.section === section),
                authorMap,
            ),
        )
        .join("\n\n");
}
