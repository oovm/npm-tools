//! Node-API bindings for commit history apply/retime and changelog helpers.

use std::env;
use std::path::{Path, PathBuf};

use nifty_history::{
    changelog::{
        collect_commits as changelog_collect_commits, collect_contributors_from_commits, detect_github_repo,
        fetch_github_user_by_login, format_tag_list, group_commits, load_author_map, lookup_github_user_by_email,
        render_contributor_wall, render_reference, resolve_range as changelog_resolve_range,
    },
    commit::{
        RetimeOptions, RetimeRootOptions, collect_commits as reword_collect_commits, dry_run_plan, export_map,
        head_ref_name, move_ref, open, parse_map, plan_rewrite, resolve_map, resolve_ref_tip, resolve_rev, run_retime,
        run_retime_root, short,
    },
    repo::find_git_root,
    validation,
};
use napi::bindgen_prelude::*;
use napi_derive::napi;

fn map_err<T>(result: nifty_history::Result<T>) -> Result<T> {
    result.map_err(|error| Error::from_reason(error.to_string()))
}

fn repo_root_from_cwd(cwd: Option<String>) -> Result<PathBuf> {
    let start = cwd
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    map_err(find_git_root(start))
}

fn open_repo_at(cwd: Option<String>) -> Result<gix::Repository> {
    let root = repo_root_from_cwd(cwd)?;
    map_err(open(&root))
}

fn normalize_ref_name(repo: &gix::Repository, ref_name: &str) -> Result<String> {
    if ref_name == "HEAD" {
        return map_err(head_ref_name(repo));
    }
    if ref_name.starts_with("refs/") {
        return Ok(ref_name.to_string());
    }
    Ok(format!("refs/heads/{ref_name}"))
}

#[napi(object)]
pub struct CommitPlannedChange {
    pub old_oid: String,
    pub old_subject: String,
    pub new_subject: String,
    pub parents_relinked: bool,
    pub message_changed: bool,
}

#[napi(object)]
pub struct CommitApplyReport {
    pub changes: Vec<CommitPlannedChange>,
    pub old_tip: String,
    pub new_tip: Option<String>,
    pub ref_name: String,
    pub dry_run: bool,
}

#[napi(object)]
pub struct CommitExportReport {
    pub count: u32,
    pub path: String,
}

#[napi(object)]
pub struct CommitExportOptions {
    pub cwd: Option<String>,
    pub base: String,
    pub r#ref: Option<String>,
    pub path: String,
}

#[napi(object)]
pub struct CommitApplyOptions {
    pub cwd: Option<String>,
    pub base: String,
    pub r#ref: Option<String>,
    pub path: String,
    pub dry_run: Option<bool>,
}

#[napi(object)]
pub struct RewordPlannedChange {
    pub old_oid: String,
    pub old_subject: String,
    pub new_subject: String,
    pub parents_relinked: bool,
    pub message_changed: bool,
}

#[napi(object)]
pub struct RewordRewriteReport {
    pub changes: Vec<RewordPlannedChange>,
    pub old_tip: String,
    pub new_tip: Option<String>,
    pub ref_name: String,
    pub dry_run: bool,
}

#[napi(object)]
pub struct RewordExportReport {
    pub count: u32,
    pub path: String,
}

#[napi(object)]
pub struct RewordExportOptions {
    pub cwd: Option<String>,
    pub base: String,
    pub r#ref: Option<String>,
    pub path: String,
}

#[napi(object)]
pub struct RewordRewriteOptions {
    pub cwd: Option<String>,
    pub base: String,
    pub r#ref: Option<String>,
    pub path: String,
    pub dry_run: Option<bool>,
}

#[napi(object)]
pub struct RetimeReport {
    pub branch: String,
    pub new_tip: String,
    pub rewritten: u32,
}

#[napi(object)]
pub struct RetimeRangeOptions {
    pub cwd: Option<String>,
    pub commit: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub branch: Option<String>,
    pub tip: Option<String>,
}

#[napi(object)]
pub struct RetimeRootOptionsNapi {
    pub cwd: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub branch: Option<String>,
    pub tip: Option<String>,
    pub message: Option<String>,
}

#[napi(object)]
pub struct ChangelogRenderReport {
    pub notes: String,
    pub version: String,
    pub from_ref: Option<String>,
    pub to_ref: String,
    pub commit_count: u32,
    pub written_path: Option<String>,
    pub range_label: String,
}

#[napi(object)]
pub struct ChangelogRenderOptions {
    pub cwd: Option<String>,
    pub version: Option<String>,
    pub from_ref: Option<String>,
    pub to_ref: Option<String>,
    pub write: Option<bool>,
    pub tags: Option<bool>,
    pub repo: Option<String>,
    pub author_map: Option<String>,
    pub releases_dir: Option<String>,
}

#[napi(object)]
pub struct ChangelogLookupOptions {
    pub cwd: Option<String>,
    pub email: Option<String>,
    pub login: Option<String>,
    pub map: Option<String>,
    pub github_token: Option<String>,
    pub fetch: Option<bool>,
}

#[napi(object)]
pub struct ChangelogGithubAuthor {
    #[napi(ts_type = "bigint")]
    pub id: Option<i64>,
    pub login: Option<String>,
}

fn to_planned_change(change: nifty_history::commit::PlannedChange) -> CommitPlannedChange {
    CommitPlannedChange {
        old_oid: short(change.old_oid),
        old_subject: change.old_subject,
        new_subject: change.new_subject,
        parents_relinked: change.parents_relinked,
        message_changed: change.message_changed,
    }
}

#[napi]
pub fn git_tools_commit_export(options: CommitExportOptions) -> Result<CommitExportReport> {
    let repo = open_repo_at(options.cwd)?;
    let ref_name = options.r#ref.as_deref().unwrap_or("HEAD");
    let exclusive_base = map_err(resolve_rev(&repo, &options.base))?;
    let tip = map_err(resolve_ref_tip(&repo, ref_name))?;
    let count = map_err(reword_collect_commits(&repo, exclusive_base, tip))?.len();
    let path = PathBuf::from(&options.path);
    map_err(export_map(&repo, exclusive_base, tip, &path))?;
    Ok(CommitExportReport {
        count: count as u32,
        path: path.display().to_string(),
    })
}

#[napi]
pub fn git_tools_commit_apply(options: CommitApplyOptions) -> Result<CommitApplyReport> {
    let repo = open_repo_at(options.cwd)?;
    let ref_name_input = options.r#ref.as_deref().unwrap_or("HEAD");
    let dry_run = options.dry_run.unwrap_or(false);
    let exclusive_base = map_err(resolve_rev(&repo, &options.base))?;
    let tip = map_err(resolve_ref_tip(&repo, ref_name_input))?;
    let chain = map_err(reword_collect_commits(&repo, exclusive_base, tip))?;
    let entries = map_err(parse_map(Path::new(&options.path)))?;
    let updates = map_err(resolve_map(entries, &chain))?;
    let ref_name = normalize_ref_name(&repo, ref_name_input)?;

    if dry_run {
        let changes = map_err(dry_run_plan(&repo, exclusive_base, tip, &updates))?;
        return Ok(CommitApplyReport {
            changes: changes.into_iter().map(to_planned_change).collect(),
            old_tip: short(tip),
            new_tip: None,
            ref_name,
            dry_run: true,
        });
    }

    let old_tip = tip;
    let (changes, new_tip) = map_err(plan_rewrite(&repo, exclusive_base, tip, &updates))?;
    if changes.is_empty() {
        return Err(Error::from_reason("no commit objects were rewritten"));
    }
    map_err(move_ref(&repo, &ref_name, new_tip, old_tip))?;
    Ok(CommitApplyReport {
        changes: changes.into_iter().map(to_planned_change).collect(),
        old_tip: short(old_tip),
        new_tip: Some(short(new_tip)),
        ref_name,
        dry_run: false,
    })
}

#[napi]
pub fn git_tools_reword_export(options: RewordExportOptions) -> Result<RewordExportReport> {
    let report = git_tools_commit_export(CommitExportOptions {
        cwd: options.cwd,
        base: options.base,
        r#ref: options.r#ref,
        path: options.path,
    })?;
    Ok(RewordExportReport {
        count: report.count,
        path: report.path,
    })
}

#[napi]
pub fn git_tools_reword_rewrite(options: RewordRewriteOptions) -> Result<RewordRewriteReport> {
    let report = git_tools_commit_apply(CommitApplyOptions {
        cwd: options.cwd,
        base: options.base,
        r#ref: options.r#ref,
        path: options.path,
        dry_run: options.dry_run,
    })?;
    Ok(RewordRewriteReport {
        changes: report
            .changes
            .into_iter()
            .map(|change| RewordPlannedChange {
                old_oid: change.old_oid,
                old_subject: change.old_subject,
                new_subject: change.new_subject,
                parents_relinked: change.parents_relinked,
                message_changed: change.message_changed,
            })
            .collect(),
        old_tip: report.old_tip,
        new_tip: report.new_tip,
        ref_name: report.ref_name,
        dry_run: report.dry_run,
    })
}

#[napi]
pub fn git_tools_retime_range(options: RetimeRangeOptions) -> Result<RetimeReport> {
    let repo = open_repo_at(options.cwd)?;
    let summary = map_err(run_retime(
        &repo,
        &RetimeOptions {
            commit: options.commit,
            start_date: options.start_date,
            end_date: options.end_date,
            branch: options.branch,
            tip: options.tip.unwrap_or_else(|| "HEAD".to_string()),
        },
    ))?;
    Ok(RetimeReport {
        branch: summary.branch,
        new_tip: short(summary.new_tip),
        rewritten: summary.rewritten as u32,
    })
}

#[napi]
pub fn git_tools_retime_root(options: RetimeRootOptionsNapi) -> Result<RetimeReport> {
    let repo = open_repo_at(options.cwd)?;
    let summary = map_err(run_retime_root(
        &repo,
        &RetimeRootOptions {
            start_date: options.start_date,
            end_date: options.end_date,
            branch: options.branch,
            tip: options.tip.unwrap_or_else(|| "HEAD".to_string()),
            message: options.message,
        },
    ))?;
    Ok(RetimeReport {
        branch: summary.branch,
        new_tip: short(summary.new_tip),
        rewritten: summary.rewritten as u32,
    })
}

fn resolve_github_repo(repo_root: &Path, override_repo: Option<&str>) -> Result<String> {
    if let Some(repo) = override_repo {
        return Ok(repo.to_string());
    }
    detect_github_repo(repo_root)
        .ok_or_else(|| Error::from_reason("could not detect GitHub repo from origin; pass --repo owner/name"))
}

fn default_author_map_path(repo_root: &Path) -> PathBuf {
    repo_root.join("documentation/maintenance/author-github.json")
}

fn default_releases_dir(repo_root: &Path) -> PathBuf {
    repo_root.join("documentation/maintenance/releases")
}

#[napi]
pub fn git_tools_changelog_render(options: ChangelogRenderOptions) -> Result<ChangelogRenderReport> {
    let repo_root = repo_root_from_cwd(options.cwd)?;

    if options.tags.unwrap_or(false) {
        let notes = map_err(format_tag_list(&repo_root))?;
        return Ok(ChangelogRenderReport {
            notes,
            version: String::new(),
            from_ref: None,
            to_ref: String::new(),
            commit_count: 0,
            written_path: None,
            range_label: String::new(),
        });
    }

    let github_repo = resolve_github_repo(&repo_root, options.repo.as_deref())?;
    let author_map_path = options
        .author_map
        .map(PathBuf::from)
        .unwrap_or_else(|| default_author_map_path(&repo_root));
    let releases_dir = options
        .releases_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| default_releases_dir(&repo_root));
    let author_map = load_author_map(&author_map_path);

    let (version, from_ref, to_ref) = map_err(changelog_resolve_range(
        &repo_root,
        options.version.as_deref(),
        options.from_ref.as_deref(),
        options.to_ref.as_deref(),
    ))?;

    let commits = map_err(changelog_collect_commits(&repo_root, from_ref.as_deref(), &to_ref))?;
    let groups = group_commits(&commits);
    let contributors = collect_contributors_from_commits(&commits, &author_map);
    let contributor_wall = render_contributor_wall(&contributors, &github_repo);
    let notes = render_reference(
        &version,
        from_ref.as_deref(),
        &to_ref,
        &groups,
        &contributor_wall,
        commits.len(),
        &author_map,
    );

    let range_label = match from_ref.as_deref() {
        Some(from) => format!("{from}..{to_ref}"),
        None => to_ref.clone(),
    };

    let mut written_path = None;
    if options.write.unwrap_or(false) {
        std::fs::create_dir_all(&releases_dir).map_err(|err| Error::from_reason(err.to_string()))?;
        let out_path = releases_dir.join(format!("v{version}.reference.md"));
        std::fs::write(&out_path, &notes).map_err(|err| Error::from_reason(err.to_string()))?;
        written_path = Some(
            out_path
                .strip_prefix(&repo_root)
                .unwrap_or(&out_path)
                .display()
                .to_string(),
        );
    }

    Ok(ChangelogRenderReport {
        notes,
        version,
        from_ref,
        to_ref,
        commit_count: commits.len() as u32,
        written_path,
        range_label,
    })
}

#[napi]
pub fn git_tools_changelog_lookup(options: ChangelogLookupOptions) -> Result<ChangelogGithubAuthor> {
    let repo_root = repo_root_from_cwd(options.cwd)?;
    let map_path = options.map.map(PathBuf::from).unwrap_or_else(|| default_author_map_path(&repo_root));
    let map = load_author_map(&map_path);
    let token = options.github_token.as_deref();
    let fetch = options.fetch.unwrap_or(false);

    let author = if let Some(email) = options.email {
        map_err(lookup_github_user_by_email(&email, &map, token, fetch))?
            .ok_or_else(|| Error::from_reason(format!("no GitHub user found for email `{email}`")))?
    } else if let Some(login) = options.login {
        map_err(fetch_github_user_by_login(&login, token))?
    } else {
        return Err(Error::from_reason(validation("lookup requires --email or --login").to_string()));
    };

    Ok(ChangelogGithubAuthor {
        id: author.id.map(|id| id as i64),
        login: author.login,
    })
}
