use nifty_publisher::{PublishFailure, PublishFailureKind};

fn classify(output: &str) -> PublishFailure {
    PublishFailure::from_npm_output(1, output)
}

#[test]
fn staged_pending_is_distinct_from_live() {
    let failure = classify(
        "npm error code E409\n\
         npm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg - Cannot publish over previously staged version 1.0.0.",
    );
    assert_eq!(failure.kind, PublishFailureKind::StagedPending);
    assert_eq!(failure.npm_code.as_deref(), Some("E409"));
}

#[test]
fn previously_published_is_live() {
    let failure = classify(
        "npm error code E403\n\
         npm error 403 Forbidden - PUT https://registry.npmjs.org/@scope%2fpkg - You cannot publish over the previously published versions: 1.0.0.",
    );
    assert_eq!(failure.kind, PublishFailureKind::Live);
    assert_eq!(failure.npm_code.as_deref(), Some("E403"));
}

#[test]
fn bare_conflict_code_is_version_conflict_not_live() {
    let failure = classify("npm error code E409\nnpm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg");
    assert_eq!(failure.kind, PublishFailureKind::VersionConflict);
    assert_eq!(failure.npm_code.as_deref(), Some("E409"));
}

#[test]
fn auth_codes_are_auth_failures() {
    assert_eq!(classify("npm error code E401\nnpm error 401 Unauthorized").kind, PublishFailureKind::Auth);
    assert_eq!(classify("npm error code ENEEDAUTH\nnpm error need auth").kind, PublishFailureKind::Auth);
    assert_eq!(classify("npm error code EOTP\nnpm error one-time password required").kind, PublishFailureKind::Auth);
}

#[test]
fn network_codes_are_network_failures() {
    assert_eq!(
        classify("npm error code ETIMEDOUT\nnpm error network request failed, reason: connect ETIMEDOUT").kind,
        PublishFailureKind::Network
    );
    assert_eq!(classify("npm error code ECONNRESET").kind, PublishFailureKind::Network);
    assert_eq!(classify("npm error code EAI_AGAIN").kind, PublishFailureKind::Network);
}

#[test]
fn dependency_codes_are_dependency_failures() {
    assert_eq!(
        classify("npm error code E404\nnpm error 404 Not Found - PUT https://registry.npmjs.org/@scope%2fmain - Not found")
            .kind,
        PublishFailureKind::DependencyBlocked
    );
    assert_eq!(
        classify("npm error code ERESOLVE\nnpm error unable to resolve dependency tree").kind,
        PublishFailureKind::DependencyBlocked
    );
}

#[test]
fn unrecognized_failure_is_unknown() {
    assert_eq!(classify("npm error code EUNKNOWN\nnpm error something else").kind, PublishFailureKind::Unknown);
    assert_eq!(classify("").kind, PublishFailureKind::Unknown);
}

#[test]
fn parses_legacy_npm_err_code_prefix() {
    let failure = classify(
        "npm ERR! code E403\n\
         npm ERR! 403 Forbidden - PUT https://registry.npmjs.org/@scope%2fpkg - You cannot publish over the previously published versions: 1.0.0.",
    );
    assert_eq!(failure.kind, PublishFailureKind::Live);
    assert_eq!(failure.npm_code.as_deref(), Some("E403"));
}

#[test]
fn error_code_line_requires_a_value() {
    // `npm error code` with no value is not a code; without other signals the failure is unknown.
    let failure = classify("npm error code\nnpm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg");
    assert_eq!(failure.npm_code, None);
    assert_eq!(failure.kind, PublishFailureKind::Unknown);
}
