use std::{fs, path::Path};

use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha1::Sha1;

use crate::Result;

type HmacSha1 = Hmac<Sha1>;

/// Workspace-root secrets file. Nifty never reads npm secrets from process env
/// (too many tools pick them up unintentionally); the file is the only source
/// besides explicit CLI overrides.
const SECRETS_FILE: &str = ".secrets/nifty.json";

/// npm 2FA：RFC 6238 TOTP 或静态 6 位码，外加可选 registry token。
#[derive(Debug, Clone, Default)]
pub struct OtpAuth {
    pub totp_secret: Option<String>,
    pub static_otp: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OtpOverrides {
    pub totp_secret: Option<String>,
    pub static_otp: Option<String>,
    pub token: Option<String>,
}

/// Shape of `.secrets/nifty.json`. Unknown keys are rejected so a typo cannot
/// silently leave a secret unread.
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SecretsFile {
    npm_token: Option<String>,
    npm_totp_secret: Option<String>,
    npm_otp: Option<String>,
}

fn oidc_ci_publish() -> bool {
    std::env::var("GITHUB_ACTIONS").ok().as_deref() == Some("true") && std::env::var("CI").ok().as_deref() == Some("true")
}

impl OtpAuth {
    /// 从 CLI 覆盖项与工作区根 `.secrets/nifty.json` 加载（不读进程 env 秘密）。
    pub fn load(workspace_root: &Path, overrides: OtpOverrides) -> Result<Self> {
        if oidc_ci_publish() {
            return Ok(OtpAuth::default());
        }
        let file = load_secrets_file(&workspace_root.join(SECRETS_FILE))?;
        Ok(OtpAuth {
            totp_secret: first_non_empty(&[
                non_empty(overrides.totp_secret),
                non_empty(file.npm_totp_secret),
                otp_as_secret(non_empty(overrides.static_otp.clone())),
                otp_as_secret(non_empty(file.npm_otp.clone())),
            ]),
            static_otp: first_non_empty(&[
                static_otp_only(non_empty(overrides.static_otp)),
                static_otp_only(non_empty(file.npm_otp)),
            ]),
            token: first_non_empty(&[non_empty(overrides.token), non_empty(file.npm_token)]),
        })
    }

    pub fn has_otp(&self) -> bool {
        self.totp_secret.is_some() || self.static_otp.is_some()
    }

    /// 每次 npm 调用前取新码（TOTP 30s 窗口）。
    pub fn current_otp(&self) -> Option<String> {
        if let Some(secret) = &self.totp_secret {
            return Some(current_totp_code(secret));
        }
        self.static_otp.clone()
    }
}

pub fn totp_code(secret: &str, at_ms: u128) -> String {
    let key = decode_base32(secret).expect("invalid TOTP secret");
    let counter = (at_ms / 1000 / 30) as u64;
    let buf = counter.to_be_bytes();

    let mut mac = HmacSha1::new_from_slice(&key).expect("HMAC accepts any key length");
    mac.update(&buf);
    let digest = mac.finalize().into_bytes();
    let offset = (digest[digest.len() - 1] & 0x0f) as usize;
    let code = ((digest[offset] & 0x7f) as u32) << 24
        | (digest[offset + 1] as u32) << 16
        | (digest[offset + 2] as u32) << 8
        | digest[offset + 3] as u32;
    format!("{:06}", code % 1_000_000)
}

fn current_totp_code(secret: &str) -> String {
    let at_ms =
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_millis()).unwrap_or(0);
    totp_code(secret, at_ms)
}

fn decode_base32(secret: &str) -> Result<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let cleaned = secret.replace([' ', '=', '-'], "").to_uppercase();
    let mut bits = String::new();
    for ch in cleaned.bytes() {
        let index = ALPHABET.iter().position(|&byte| byte == ch);
        let index = index.ok_or_else(|| "invalid base32 in TOTP secret".to_string())?;
        bits.push_str(&format!("{:05b}", index));
    }
    let mut bytes = Vec::new();
    for chunk in bits.as_bytes().chunks(8) {
        if chunk.len() < 8 {
            break;
        }
        let value = u8::from_str_radix(std::str::from_utf8(chunk).unwrap_or("0"), 2).map_err(|err| err.to_string())?;
        bytes.push(value);
    }
    if bytes.is_empty() {
        return Err("TOTP secret decoded empty".into());
    }
    Ok(bytes)
}

/// Missing file is fine (CLI overrides only); a malformed file is an error.
fn load_secrets_file(path: &Path) -> Result<SecretsFile> {
    match fs::read_to_string(path) {
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(SecretsFile::default()),
        Err(err) => Err(format!("failed to read {}: {err}", path.display())),
        Ok(text) => serde_json::from_str(&text).map_err(|err| format!("invalid {}: {err}", path.display())),
    }
}

fn non_empty(value: Option<String>) -> Option<String> {
    value.map(|text| text.trim().to_string()).filter(|text| !text.is_empty())
}

fn otp_as_secret(value: Option<String>) -> Option<String> {
    value.filter(|text| !is_six_digit_otp(text))
}

fn static_otp_only(value: Option<String>) -> Option<String> {
    value.filter(|text| is_six_digit_otp(text))
}

fn is_six_digit_otp(value: &str) -> bool {
    value.trim().len() == 6 && value.trim().chars().all(|ch| ch.is_ascii_digit())
}

fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values.iter().find_map(|value| value.clone())
}

#[cfg(test)]
mod tests {
    use super::{decode_base32, load_secrets_file, totp_code};

    #[test]
    fn decodes_base32_secret() {
        let bytes = decode_base32("JBSWY3DPEHPK3PXP").expect("decode");
        assert!(!bytes.is_empty());
    }

    #[test]
    fn totp_is_six_digits() {
        let code = totp_code("JBSWY3DPEHPK3PXP", 59_000);
        assert_eq!(code.len(), 6);
        assert!(code.chars().all(|ch| ch.is_ascii_digit()));
    }

    #[test]
    fn missing_secrets_file_is_empty() {
        let path = std::path::Path::new(".secrets").join("definitely-missing-nifty.json");
        let file = load_secrets_file(&path).expect("missing file loads as default");
        assert!(file.npm_token.is_none());
        assert!(file.npm_totp_secret.is_none());
        assert!(file.npm_otp.is_none());
    }

    #[test]
    fn parses_camel_case_secrets() {
        let dir = std::env::temp_dir().join(format!("nifty-secrets-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("nifty.json");
        std::fs::write(&path, r#"{"npmToken":"npm_x","npmTotpSecret":"JBSW","npmOtp":"123456"}"#).expect("write");
        let file = load_secrets_file(&path).expect("parse");
        assert_eq!(file.npm_token.as_deref(), Some("npm_x"));
        assert_eq!(file.npm_totp_secret.as_deref(), Some("JBSW"));
        assert_eq!(file.npm_otp.as_deref(), Some("123456"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_unknown_secret_keys() {
        let dir = std::env::temp_dir().join(format!("nifty-secrets-typo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("nifty.json");
        std::fs::write(&path, r#"{"npmTotp":"oops"}"#).expect("write");
        let err = load_secrets_file(&path).expect_err("typo key must fail");
        assert!(err.contains("npmTotp"), "error should name the bad key: {err}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
