//! Failure classes shared by every Nifty backend that resolves a revision to a commit.
//!
//! The wording lives here so the gix-backed and git-CLI-backed paths cannot drift apart.
//! `Ambiguous` and `Other` keep the backend message verbatim: only that message carries the
//! candidate list and context that a flat sentence cannot reproduce.

use std::fmt::{Display, Formatter, Result as FmtResult};

/// Why a revision spec could not be resolved to a commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefError {
    /// Nothing in the repository matches the spec.
    NotFound {
        /// The revision spec as the caller wrote it.
        reference: String,
    },
    /// The spec resolves to an object that is not a commit, such as a tag pointing at a tree.
    NotCommit {
        /// The revision spec as the caller wrote it.
        reference: String,
    },
    /// The spec matches more than one object.
    Ambiguous {
        /// The revision spec as the caller wrote it.
        reference: String,
        /// Backend message, including the candidate list.
        detail: String,
    },
    /// Any other backend failure, kept verbatim.
    Other {
        /// Backend message.
        detail: String,
    },
}

impl Display for RefError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            RefError::NotFound { reference } => write!(f, "ref does not exist: {reference:?}"),
            RefError::NotCommit { reference } => write!(f, "ref is not a commit: {reference:?}"),
            RefError::Ambiguous { detail, .. } | RefError::Other { detail } => f.write_str(detail),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RefError;

    #[test]
    fn missing_and_non_commit_refs_stay_distinct() {
        let missing = RefError::NotFound { reference: "no-such-ref".to_string() };
        let not_commit = RefError::NotCommit { reference: "treetag".to_string() };
        assert_eq!(missing.to_string(), "ref does not exist: \"no-such-ref\"");
        assert_eq!(not_commit.to_string(), "ref is not a commit: \"treetag\"");
    }

    #[test]
    fn ambiguous_keeps_the_backend_candidate_list() {
        let detail = "Short id abc123 is ambiguous. Candidates are:\n\tdeadbeef commit now \"title\"";
        let err = RefError::Ambiguous { reference: "abc123".to_string(), detail: detail.to_string() };
        assert_eq!(err.to_string(), detail);
    }

    #[test]
    fn other_keeps_the_backend_message() {
        let detail = "The rev-spec is malformed and misses a ref name";
        let err = RefError::Other { detail: detail.to_string() };
        assert_eq!(err.to_string(), detail);
    }
}
