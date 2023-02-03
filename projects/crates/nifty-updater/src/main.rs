use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use nifty_linter::{run_lint, LintOptions};
use nifty_uploader::{action::parse_target, run_upload, UploadOptions};
use nifty_updater::{run_update, UpdateOptions};

#[derive(Parser)]
#[command(name = "nifty", author, version, about = "Nifty tooling for hybrid cargo + npm projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Update cargo and npm dependencies
    Update(UpdateArgs),
    /// Lint gitmoji commit conventions (rule-based)
    Lint(LintArgs),
    /// Lint and exit with failure when errors are found
    Check(LintArgs),
    /// Upload artifacts to GitHub Release and/or GitHub Pages
    Upload(UploadArgs),
}

#[derive(Parser, Clone)]
struct UpdateArgs {
    #[arg(short = 'i', long = "interactive")]
    interactive: bool,
    #[arg(short = 'C', long = "cwd")]
    cwd: Option<PathBuf>,
}

#[derive(Parser, Clone)]
struct LintArgs {
    #[arg(long)]
    from: Option<String>,
    #[arg(long)]
    to: Option<String>,
    #[arg(short, long = "subject")]
    subjects: Vec<String>,
    #[arg(short = 'C', long = "cwd")]
    cwd: Option<PathBuf>,
}

#[derive(Parser, Clone)]
struct UploadArgs {
    /// Upload to GitHub Release
    #[arg(long)]
    release: bool,
    /// Upload to GitHub Pages (`gh-pages` branch)
    #[arg(long)]
    pages: bool,
    /// Upload to both release and pages
    #[arg(long)]
    both: bool,
    /// Directory to upload
    #[arg(short, long = "dir", default_value = "dist")]
    dir: PathBuf,
    /// Release tag
    #[arg(short, long)]
    tag: Option<String>,
    /// GitHub repo `owner/name`
    #[arg(long)]
    repo: Option<String>,
    /// GitHub token
    #[arg(long)]
    token: Option<String>,
    /// Release title
    #[arg(long)]
    name: Option<String>,
    /// Release notes body
    #[arg(long)]
    notes: Option<String>,
    /// Release notes file
    #[arg(long = "notes-file")]
    notes_file: Option<PathBuf>,
    /// Create draft release
    #[arg(long)]
    draft: bool,
    /// Generate release notes from git history
    #[arg(long = "generate-notes", default_value_t = true)]
    generate_notes: bool,
    /// Read `GITHUB_*` environment defaults (for Actions)
    #[arg(long = "github-action")]
    github_action: bool,
    #[arg(short = 'C', long = "cwd")]
    cwd: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Update(args) => match run_update(UpdateOptions {
            interactive: args.interactive,
            cwd: args.cwd,
        }) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("error: {err}");
                ExitCode::FAILURE
            }
        },
        Commands::Lint(args) => run_lint_command(args, false),
        Commands::Check(args) => run_lint_command(args, true),
        Commands::Upload(args) => run_upload_command(args),
    }
}

fn run_lint_command(args: LintArgs, fail_on_error: bool) -> ExitCode {
    let options = LintOptions {
        repo_root: args.cwd,
        from_ref: args.from,
        to_ref: args.to,
        subjects: args.subjects,
        rules: None,
    };

    match run_lint(options) {
        Ok(report) => {
            report.print_human();
            if fail_on_error && report.has_errors() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_upload_command(args: UploadArgs) -> ExitCode {
    let target = match parse_target(args.pages, args.release, args.both) {
        Ok(target) => target,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    match run_upload(UploadOptions {
        cwd: args.cwd,
        repo: args.repo,
        token: args.token,
        tag: args.tag,
        release_name: args.name,
        notes: args.notes,
        notes_file: args.notes_file,
        dir: args.dir,
        target,
        github_action: args.github_action,
        draft: args.draft,
        generate_notes: args.generate_notes,
    }) {
        Ok(report) => {
            if !report.release_assets.is_empty() {
                println!("upload: {} release asset(s)", report.release_assets.len());
            }
            if let Some(branch) = report.pages_branch {
                println!("upload: deployed to {branch}");
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
