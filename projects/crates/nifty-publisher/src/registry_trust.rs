use std::path::Path;

use serde_json::{Value, json};

use crate::{
    Result,
    cache::{TrustExpect, classify_configs},
    npmrc::NpmRc,
    otp::OtpAuth,
};

const USER_AGENT: &str = "nifty-publisher (https://github.com/oovm/npm-tools)";

pub struct RegistryTrustClient {
    npmrc: NpmRc,
    auth: OtpAuth,
}

impl RegistryTrustClient {
    pub fn from_workspace(auth: &OtpAuth, workspace_root: &Path) -> Result<Self> {
        Ok(Self { npmrc: NpmRc::load_merged(workspace_root), auth: auth.clone() })
    }

    pub fn list(&self, package: &str) -> Result<Vec<Value>> {
        let registry = self.npmrc.registry_for_package(package);
        let token = resolve_registry_token(&self.auth, &self.npmrc, package)?;
        let path = trust_path(package);
        let (status, body) = registry_request(&registry, "GET", &path, &token, self.auth.current_otp().as_deref(), None)?;
        if status == 404 {
            return Ok(Vec::new());
        }
        if !(200..300).contains(&status) {
            return Err(format!("registry trust list failed for {package} ({status}): {body}"));
        }
        parse_trust_configs(&body)
    }

    pub fn create(&self, package: &str, expect: &TrustExpect) -> Result<TrustCreateOutcome> {
        let registry = self.npmrc.registry_for_package(package);
        let token = resolve_registry_token(&self.auth, &self.npmrc, package)?;
        let path = trust_path(package);
        let body = trust_create_body(expect);
        let (status, response_body) =
            registry_request(&registry, "POST", &path, &token, self.auth.current_otp().as_deref(), Some(&body))?;
        match status {
            200 | 201 => Ok(TrustCreateOutcome::Created),
            409 => Ok(TrustCreateOutcome::AlreadyExists),
            _ => Err(format!("registry trust create failed for {package} ({status}): {response_body}")),
        }
    }
}

pub enum TrustCreateOutcome {
    Created,
    AlreadyExists,
}

pub fn resolve_registry_token(auth: &OtpAuth, npmrc: &NpmRc, package: &str) -> Result<String> {
    if let Some(token) = auth.token.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(token.to_string());
    }
    if let Some(token) = npmrc.token_for_package(package) {
        return Ok(token);
    }
    Err(
        "registry trust requires NPM_TOKEN in env or .env.placeholder.local, or an npm login token in ~/.npmrc / project .npmrc"
            .into(),
    )
}

fn trust_path(package: &str) -> String {
    format!("/-/package/{}/trust", urlencoding::encode(package))
}

fn trust_create_body(expect: &TrustExpect) -> Value {
    json!([{
        "type": "github",
        "claims": {
            "repository": expect.repo,
            "workflow_ref": { "file": expect.file },
            "environment": expect.env,
        },
        "permissions": ["createPackage"],
    }])
}

pub fn parse_trust_configs(body: &str) -> Result<Vec<Value>> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let json = extract_json_payload(trimmed);
    let data: Value = serde_json::from_str(json).map_err(|err| err.to_string())?;
    if let Some(array) = data.as_array() {
        return Ok(array.clone());
    }
    if let Some(array) = data.get("configurations").and_then(Value::as_array) {
        return Ok(array.clone());
    }
    if let Some(array) = data.get("items").and_then(Value::as_array) {
        return Ok(array.clone());
    }
    if data.is_object() {
        return Ok(vec![data]);
    }
    Ok(Vec::new())
}

fn extract_json_payload(stdout: &str) -> &str {
    if let Some(start) = stdout.find('[') {
        return stdout[start..].trim();
    }
    if let Some(start) = stdout.find('{') {
        return stdout[start..].trim();
    }
    stdout.trim()
}

fn registry_request(
    registry: &str,
    method: &str,
    path: &str,
    token: &str,
    otp: Option<&str>,
    body: Option<&Value>,
) -> Result<(u16, String)> {
    let base = registry.trim_end_matches('/');
    let url = format!("{base}{path}");
    let payload = body.map(|value| serde_json::to_vec(value).map_err(|err| err.to_string()));
    let payload = match payload {
        Some(result) => Some(result?),
        None => None,
    };

    let mut request = ureq::request(method, &url)
        .set("Accept", "application/json")
        .set("Authorization", &format!("Bearer {token}"))
        .set("User-Agent", USER_AGENT);
    if let Some(code) = otp {
        request = request.set("npm-otp", code);
    }
    let response = if let Some(bytes) = &payload {
        request.set("Content-Type", "application/json").send_bytes(bytes)
    }
    else {
        request.call()
    }
    .map_err(|err| format!("registry request failed ({method} {url}): {err}"))?;

    let status = response.status();
    let body = response.into_string().map_err(|err| err.to_string())?;
    Ok((status, body))
}

pub fn trust_already_matches(configs: &[Value], expect: &TrustExpect) -> bool {
    classify_configs(configs, expect).matches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::npmrc::NpmRc;

    #[test]
    fn builds_github_trust_body() {
        let body = trust_create_body(&TrustExpect {
            repo: "yy-database/yydb".to_string(),
            file: "release-npm.yml".to_string(),
            env: "NPM_PUBLISH".to_string(),
        });
        let entry = body.as_array().and_then(|items| items.first()).expect("array");
        assert_eq!(entry.get("type").and_then(Value::as_str), Some("github"));
        assert_eq!(entry.pointer("/claims/repository").and_then(Value::as_str), Some("yy-database/yydb"));
    }

    #[test]
    fn resolves_token_from_npmrc_for_package() {
        let npmrc = NpmRc::from_text("//registry.npmjs.org/:_authToken=npm_from_npmrc\n");
        let auth = OtpAuth::default();
        let token = resolve_registry_token(&auth, &npmrc, "@yydb/yydb").expect("token");
        assert_eq!(token, "npm_from_npmrc");
    }

    #[test]
    fn parses_trust_list_array() {
        let configs = parse_trust_configs(
            r#"[{"type":"github","claims":{"repository":"oovm/npm-tools","workflow_ref":{"file":"publish-npm.yml"},"environment":"NPM_PUBLISH"}}]"#,
        )
        .expect("parse");
        assert_eq!(configs.len(), 1);
        assert!(trust_already_matches(
            &configs,
            &TrustExpect {
                repo: "oovm/npm-tools".to_string(),
                file: "publish-npm.yml".to_string(),
                env: "NPM_PUBLISH".to_string(),
            },
        ));
    }
}
