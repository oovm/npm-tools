use std::path::Path;

use nifty_git::detect_github_repo;

use crate::{
    cache::TrustExpect,
    trust::{TRUST_ENV, TRUST_FILE, TRUST_REPO},
};

#[derive(Debug, Clone, Default)]
pub struct TrustExpectInput {
    pub repo: Option<String>,
    pub file: Option<String>,
    pub env: Option<String>,
}

pub fn resolve_trust_expect(root: &Path, input: Option<&TrustExpectInput>) -> TrustExpect {
    let mut repo = TRUST_REPO.to_string();
    let mut file = TRUST_FILE.to_string();
    let mut env = TRUST_ENV.to_string();

    if let Ok(Some(detected)) = detect_github_repo(root) {
        repo = detected;
    }

    if let Ok(value) = std::env::var("NIFTY_TRUST_REPO") {
        let value = value.trim();
        if !value.is_empty() {
            repo = value.to_string();
        }
    }
    file = env_or_default("NIFTY_TRUST_FILE", &file);
    env = env_or_default("NIFTY_TRUST_ENV", &env);

    if let Some(input) = input {
        if let Some(value) = non_empty(input.repo.as_deref()) {
            repo = value;
        }
        if let Some(value) = non_empty(input.file.as_deref()) {
            file = value;
        }
        if let Some(value) = non_empty(input.env.as_deref()) {
            env = value;
        }
    }

    TrustExpect { repo, file, env }
}

fn non_empty(value: Option<&str>) -> Option<String> {
    value.map(str::trim).filter(|value| !value.is_empty()).map(str::to_string)
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_trust_overrides_git_and_env() {
        std::env::set_var("NIFTY_TRUST_REPO", "env/repo");
        std::env::set_var("NIFTY_TRUST_FILE", "env.yml");
        std::env::set_var("NIFTY_TRUST_ENV", "ENV_ONLY");

        let input = TrustExpectInput {
            repo: Some("yy-database/yydb".to_string()),
            file: Some("release-npm.yml".to_string()),
            env: None,
        };
        let expect = resolve_trust_expect(Path::new("."), Some(&input));
        assert_eq!(expect.repo, "yy-database/yydb");
        assert_eq!(expect.file, "release-npm.yml");
        assert_eq!(expect.env, "ENV_ONLY");

        std::env::remove_var("NIFTY_TRUST_REPO");
        std::env::remove_var("NIFTY_TRUST_FILE");
        std::env::remove_var("NIFTY_TRUST_ENV");
    }
}
