//! Structured `npm publish` outcomes.
//!
//! npm exposes a machine-readable error code (`npm error code E409`, legacy
//! `npm ERR! code E409`) and a registry HTTP status in the same blob. The
//! classifier keys off those codes first and only falls back to the registry's
//! exact conflict sentences where npm has no distinct code for two outcomes.

use crate::cache::PlaceholderCache;

/// Why a single `npm publish` invocation did not succeed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublishFailureKind {
    /// The version is already on the live registry.
    Live,
    /// The registry holds a staged (unpromoted) version. Never treat as live.
    StagedPending,
    /// A version conflict that is neither live nor staged (for example a bare `E409`
    /// without a registry conflict sentence). Must not be cached as live.
    VersionConflict,
    /// Authentication, authorization, or OTP failure.
    Auth,
    /// Registry unreachable or timed out.
    Network,
    /// npm refused the publish because of a dependency/engine resolution problem.
    DependencyBlocked,
    /// Anything npm did not describe with a recognized code.
    Unknown,
}

impl PublishFailureKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "already live",
            Self::StagedPending => "staged pending",
            Self::VersionConflict => "version conflict",
            Self::Auth => "auth failure",
            Self::Network => "network failure",
            Self::DependencyBlocked => "dependency blocked",
            Self::Unknown => "unknown failure",
        }
    }
}

/// One categorized `npm publish` failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishFailure {
    pub kind: PublishFailureKind,
    pub exit_status: i32,
    pub npm_code: Option<String>,
    pub message: String,
}

impl PublishFailure {
    /// A failure that happened before npm could report a registry response
    /// (manifest IO, spawn failure, and similar local errors).
    pub fn local(message: impl Into<String>) -> Self {
        Self { kind: PublishFailureKind::Unknown, exit_status: 1, npm_code: None, message: message.into() }
    }

    /// Parse npm's structured error code from combined stdout/stderr and classify.
    pub fn from_npm_output(exit_status: i32, output: &str) -> Self {
        let npm_code = parse_npm_error_code(output);
        let kind = classify(exit_status, npm_code.as_deref(), output);
        let message = output.trim().to_string();
        Self { kind, exit_status, npm_code, message }
    }

    /// Replace the raw output with the caller's human-readable rendering.
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    pub fn summary(&self) -> String {
        let code = self.npm_code.as_deref().map(|code| format!(" (npm code {code})")).unwrap_or_default();
        let message = self.message.split_whitespace().collect::<Vec<_>>().join(" ");
        format!("{}{}: {message}", self.kind.label(), code)
    }
}

/// Result of publishing one package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishAttempt {
    Published,
    AlreadyLive,
    StagedPending,
    Failed(PublishFailure),
}

impl PublishAttempt {
    /// Map a classified failure onto the matching batch outcome.
    pub fn from_failure(failure: PublishFailure) -> Self {
        match failure.kind {
            PublishFailureKind::Live => Self::AlreadyLive,
            PublishFailureKind::StagedPending => Self::StagedPending,
            _ => Self::Failed(failure),
        }
    }
}

/// One failed package in a batch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageFailure {
    pub name: String,
    pub version: String,
    pub failure: PublishFailure,
}

/// Accumulated batch results. Recording a failure never stops the batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PublishBatch {
    pub published: Vec<String>,
    pub already_live: Vec<String>,
    /// Staged-only packages. They are deliberately absent from the live version cache.
    pub staged: Vec<String>,
    pub failures: Vec<PackageFailure>,
}

impl PublishBatch {
    /// Record one package outcome. Returns whether the live version cache changed.
    ///
    /// Only [`PublishAttempt::Published`] and [`PublishAttempt::AlreadyLive`] touch the live
    /// version cache. Staged and failed outcomes are recorded in the batch report only.
    pub fn record(
        &mut self,
        cache: &mut PlaceholderCache,
        name: &str,
        version: &str,
        attempt: PublishAttempt,
        dry_run: bool,
    ) -> bool {
        match attempt {
            PublishAttempt::Published => {
                self.published.push(name.to_string());
                if dry_run {
                    return false;
                }
                cache.record_version(name, version);
                true
            }
            PublishAttempt::AlreadyLive => {
                self.already_live.push(name.to_string());
                if dry_run {
                    return false;
                }
                cache.record_version(name, version);
                true
            }
            PublishAttempt::StagedPending => {
                self.staged.push(name.to_string());
                false
            }
            PublishAttempt::Failed(failure) => {
                self.failures.push(PackageFailure { name: name.to_string(), version: version.to_string(), failure });
                false
            }
        }
    }

    /// Aggregated failure text, or `None` when every package succeeded or was skipped.
    pub fn failure_summary(&self) -> Option<String> {
        if self.failures.is_empty() {
            return None;
        }
        let lines = self
            .failures
            .iter()
            .map(|failure| format!("  - {}@{}: {}", failure.name, failure.version, failure.failure.summary()))
            .collect::<Vec<_>>()
            .join("\n");
        Some(format!("publish failed for {} package(s):\n{lines}", self.failures.len()))
    }
}

/// Read `npm error code X` (npm >= 8) or `npm ERR! code X` (legacy) from CLI output.
fn parse_npm_error_code(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let rest = line.trim().strip_prefix("npm error code ").or_else(|| line.trim().strip_prefix("npm ERR! code "))?;
        let code = rest.split_whitespace().next()?.trim();
        if code.is_empty() { None } else { Some(code.to_ascii_uppercase()) }
    })
}

fn classify(_exit_status: i32, npm_code: Option<&str>, output: &str) -> PublishFailureKind {
    // The registry uses the same conflict code (`E409`) for live and staged over-publish
    // and never returns a distinct code for staged. These two exact registry sentences are
    // therefore the only trustworthy discriminator between live and staged conflicts.
    const LIVE_MARKERS: [&str; 3] =
        ["cannot publish over the previously published", "previously published version", "already been published"];
    const STAGED_MARKERS: [&str; 2] = ["previously staged", "previously-staged"];
    const AUTH_CODES: [&str; 6] = ["ENEEDAUTH", "E401", "E403", "EOTP", "EPERM", "EACCES"];
    const NETWORK_CODES: [&str; 7] =
        ["ECONNRESET", "ECONNREFUSED", "ETIMEDOUT", "ENOTFOUND", "EAI_AGAIN", "EPIPE", "ERR_SOCKET_TIMEOUT"];
    const DEPENDENCY_CODES: [&str; 5] = ["ERESOLVE", "ETARGET", "EBADENGINE", "EPEERINVALID", "E404"];
    const CONFLICT_CODES: [&str; 2] = ["E409", "EPUBLISHCONFLICT"];

    let lower = output.to_ascii_lowercase();
    if STAGED_MARKERS.iter().any(|marker| lower.contains(marker)) {
        return PublishFailureKind::StagedPending;
    }
    if LIVE_MARKERS.iter().any(|marker| lower.contains(marker)) || lower.contains("version already exists") {
        return PublishFailureKind::Live;
    }
    if let Some(code) = npm_code {
        if CONFLICT_CODES.contains(&code) {
            return PublishFailureKind::VersionConflict;
        }
        if AUTH_CODES.contains(&code) {
            return PublishFailureKind::Auth;
        }
        if NETWORK_CODES.contains(&code) {
            return PublishFailureKind::Network;
        }
        if DEPENDENCY_CODES.contains(&code) {
            return PublishFailureKind::DependencyBlocked;
        }
    }
    // Legacy npm sometimes omits the code line but still reports the registry PUT response.
    // A `404 Not Found - PUT` during publish means the referenced dependency is unavailable.
    if lower.contains("not found") && lower.contains("put") {
        return PublishFailureKind::DependencyBlocked;
    }
    PublishFailureKind::Unknown
}
