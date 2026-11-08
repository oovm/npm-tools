use std::path::Path;

use nifty_git::detect_github_repo;

use crate::{
    Result,
    cache::TrustExpect,
    trust::{TRUST_ENV, TRUST_FILE, TRUST_REPO},
};

#[derive(Debug, Clone, Default)]
pub struct TrustExpectInput {
    pub repo: Option<String>,
    pub file: Option<String>,
    pub env: Option<String>,
}

pub fn resolve_trust_expect(root: &Path, input: Option<&TrustExpectInput>) -> Result<TrustExpect> {
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
        repo = explicit_value(input.repo.as_deref(), "trust.npm.repo", repo)?;
        file = explicit_value(input.file.as_deref(), "trust.npm.file", file)?;
        env = explicit_value(input.env.as_deref(), "trust.npm.environment", env)?;
    }

    Ok(TrustExpect { repo, file, env })
}

/// Replace the env-derived default with an explicit config value.
///
/// `TrustExpect` is matched by substring (`raw.contains(expect.env)`) and by exact claims,
/// so an empty value would match every registry trust config. Falling back silently would
/// both hide the explicit empty value and produce that overly loose contract.
fn explicit_value(value: Option<&str>, field: &str, current: String) -> Result<String> {
    match value {
        None => Ok(current),
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Err(format!(
                    "`{field}` is an empty string, which would match every trust config. Remove it to inherit the default"
                ))
            }
            else {
                Ok(trimmed.to_string())
            }
        }
    }
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
        let expect = resolve_trust_expect(Path::new("."), Some(&input)).expect("resolve trust expect");
        assert_eq!(expect.repo, "yy-database/yydb");
        assert_eq!(expect.file, "release-npm.yml");
        assert_eq!(expect.env, "ENV_ONLY");

        let input = TrustExpectInput { repo: None, file: None, env: Some(" CONFIG_ENV ".to_string()) };
        let expect = resolve_trust_expect(Path::new("."), Some(&input)).expect("resolve trust expect");
        assert_eq!(expect.env, "CONFIG_ENV");

        std::env::remove_var("NIFTY_TRUST_REPO");
        std::env::remove_var("NIFTY_TRUST_FILE");
        std::env::remove_var("NIFTY_TRUST_ENV");
    }

    #[test]
    fn explicit_empty_env_is_rejected_instead_of_rewritten() {
        let input = TrustExpectInput { repo: None, file: None, env: Some(String::new()) };
        let error = resolve_trust_expect(Path::new("."), Some(&input)).expect_err("empty env must be rejected");
        assert!(error.contains("trust.npm.environment"), "error must name the offending field: {error}");
    }
}
