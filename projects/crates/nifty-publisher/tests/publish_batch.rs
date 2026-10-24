use nifty_publisher::{PlaceholderCache, PublishAttempt, PublishBatch, PublishFailure, TrustExpect};

fn empty_cache() -> PlaceholderCache {
    PlaceholderCache::empty(TrustExpect {
        repo: "oovm/npm-tools".to_string(),
        file: "publish-npm.yml".to_string(),
        env: "NPM_PUBLISH".to_string(),
    })
}

fn failure(kind_output: &str) -> PublishFailure {
    PublishFailure::from_npm_output(1, kind_output)
}

#[test]
fn batch_continues_after_independent_failure_and_aggregates() {
    let mut cache = empty_cache();
    let mut batch = PublishBatch::default();

    assert!(batch.record(&mut cache, "@scope/first", "1.0.0", PublishAttempt::Published, false));
    assert!(!batch.record(
        &mut cache,
        "@scope/broken",
        "1.0.0",
        PublishAttempt::Failed(failure("npm error code E401\nnpm error 401 Unauthorized")),
        false
    ));
    assert!(!batch.record(&mut cache, "@scope/staged", "1.0.0", PublishAttempt::StagedPending, false));
    assert!(batch.record(&mut cache, "@scope/last", "1.0.0", PublishAttempt::Published, false));

    assert_eq!(batch.published, vec!["@scope/first".to_string(), "@scope/last".to_string()]);
    assert_eq!(batch.staged, vec!["@scope/staged".to_string()]);
    assert_eq!(batch.failures.len(), 1);
    assert_eq!(batch.failures[0].name, "@scope/broken");

    let summary = batch.failure_summary().expect("aggregated failure");
    assert!(summary.contains("publish failed for 1 package(s)"));
    assert!(summary.contains("@scope/broken"));
}

#[test]
fn staged_pending_never_writes_live_cache() {
    let mut cache = empty_cache();
    let mut batch = PublishBatch::default();

    assert!(!batch.record(&mut cache, "@scope/staged", "1.0.0", PublishAttempt::StagedPending, false));
    assert_eq!(cache.cached_version("@scope/staged"), None);
    assert!(batch.failure_summary().is_none());
    assert_eq!(batch.staged, vec!["@scope/staged".to_string()]);
}

#[test]
fn version_conflict_never_writes_live_cache() {
    let mut cache = empty_cache();
    let mut batch = PublishBatch::default();
    let conflict = failure("npm error code E409\nnpm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg");

    assert!(!batch.record(&mut cache, "@scope/conflict", "1.0.0", PublishAttempt::Failed(conflict), false));
    assert_eq!(cache.cached_version("@scope/conflict"), None);
    assert_eq!(batch.failures.len(), 1);
}

#[test]
fn published_and_already_live_write_live_cache() {
    let mut cache = empty_cache();
    let mut batch = PublishBatch::default();

    assert!(batch.record(&mut cache, "@scope/published", "1.0.0", PublishAttempt::Published, false));
    assert!(batch.record(&mut cache, "@scope/live", "1.0.0", PublishAttempt::AlreadyLive, false));

    assert_eq!(cache.cached_version("@scope/published"), Some("1.0.0"));
    assert_eq!(cache.cached_version("@scope/live"), Some("1.0.0"));
    assert_eq!(batch.already_live, vec!["@scope/live".to_string()]);
}

#[test]
fn dry_run_does_not_write_live_cache() {
    let mut cache = empty_cache();
    let mut batch = PublishBatch::default();

    assert!(!batch.record(&mut cache, "@scope/published", "1.0.0", PublishAttempt::Published, true));
    assert!(!batch.record(&mut cache, "@scope/live", "1.0.0", PublishAttempt::AlreadyLive, true));

    assert_eq!(cache.cached_version("@scope/published"), None);
    assert_eq!(cache.cached_version("@scope/live"), None);
}

#[test]
fn failed_attempt_maps_from_classified_failure() {
    let live = PublishAttempt::from_failure(failure(
        "npm error code E403\n\
         npm error 403 Forbidden - PUT https://registry.npmjs.org/@scope%2fpkg - You cannot publish over the previously published versions: 1.0.0.",
    ));
    assert_eq!(live, PublishAttempt::AlreadyLive);

    let staged = PublishAttempt::from_failure(failure(
        "npm error code E409\nnpm error 409 Conflict - PUT https://registry.npmjs.org/@scope%2fpkg - Cannot publish over previously staged version 1.0.0.",
    ));
    assert_eq!(staged, PublishAttempt::StagedPending);

    let auth = PublishAttempt::from_failure(failure("npm error code E401\nnpm error 401 Unauthorized"));
    assert!(matches!(auth, PublishAttempt::Failed(_)));
}
