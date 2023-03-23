use std::path::Path;

use nifty_git::detect_github_repo;

use crate::cache::TrustExpect;
use crate::trust::{TRUST_ENV, TRUST_FILE, TRUST_REPO};

pub fn resolve_trust_expect(root: &Path) -> TrustExpect {
    if let Ok(repo) = std::env::var("NIFTY_TRUST_REPO") {
        let repo = repo.trim();
        if !repo.is_empty() {
            return TrustExpect {
                repo: repo.to_string(),
                file: env_or_default("NIFTY_TRUST_FILE", TRUST_FILE),
                env: env_or_default("NIFTY_TRUST_ENV", TRUST_ENV),
            };
        }
    }
    if let Ok(Some(repo)) = detect_github_repo(root) {
        return TrustExpect {
            repo,
            file: TRUST_FILE.to_string(),
            env: TRUST_ENV.to_string(),
        };
    }
    TrustExpect {
        repo: TRUST_REPO.to_string(),
        file: TRUST_FILE.to_string(),
        env: TRUST_ENV.to_string(),
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}
