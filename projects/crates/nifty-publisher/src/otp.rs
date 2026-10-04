use std::{fs, path::Path};

use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::Result;

type HmacSha1 = Hmac<Sha1>;

const ENV_FILE_NAME: &str = ".env.placeholder.local";

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

fn oidc_ci_publish() -> bool {
    std::env::var("GITHUB_ACTIONS").ok().as_deref() == Some("true") && std::env::var("CI").ok().as_deref() == Some("true")
}

impl OtpAuth {
    /// 从 CLI 覆盖项、进程环境、仓库根 `.env.placeholder.local` 加载。
    pub fn load(workspace_root: &Path, overrides: OtpOverrides) -> Self {
        if oidc_ci_publish() {
            return OtpAuth::default();
        }
        let file_env = load_local_env(&workspace_root.join(ENV_FILE_NAME));
        OtpAuth {
            totp_secret: first_non_empty(&[
                overrides.totp_secret,
                env_var("NPM_TOTP_SECRET"),
                env_var("TOTP_SECRET"),
                file_env.get("NPM_TOTP_SECRET").cloned(),
                file_env.get("TOTP_SECRET").cloned(),
                otp_as_secret(overrides.static_otp.clone()),
                otp_as_secret(env_var("NPM_OTP")),
                otp_as_secret(file_env.get("NPM_OTP").cloned()),
                otp_as_secret(env_var("OTP")),
                otp_as_secret(file_env.get("OTP").cloned()),
            ]),
            static_otp: first_non_empty(&[
                static_otp_only(overrides.static_otp),
                static_otp_only(env_var("NPM_OTP")),
                static_otp_only(file_env.get("NPM_OTP").cloned()),
                static_otp_only(env_var("OTP")),
                static_otp_only(file_env.get("OTP").cloned()),
            ]),
            token: first_non_empty(&[
                overrides.token,
                env_var("NPM_TOKEN"),
                file_env.get("NPM_TOKEN").cloned(),
                env_var("TOKEN"),
                file_env.get("TOKEN").cloned(),
            ]),
        }
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

fn load_local_env(path: &Path) -> std::collections::BTreeMap<String, String> {
    let mut out = std::collections::BTreeMap::new();
    let text = fs::read_to_string(path).unwrap_or_default();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=')
        else {
            continue;
        };
        let key = key.trim().trim_start_matches("export ").trim();
        let mut value = value.trim().to_string();
        if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
            value = value[1..value.len() - 1].to_string();
        }
        out.insert(key.to_string(), value);
    }
    out
}

fn env_var(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
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
    use super::{decode_base32, totp_code};

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
}
