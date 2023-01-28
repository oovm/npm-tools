use std::path::{Path, PathBuf};

/// Config file names searched from the project root upward (`nifty.config.ts` first).
pub const CONFIG_FILE_NAMES: &[&str] = &[
    "nifty.config.ts",
    "nifty.config.js",
    "nifty.config.mjs",
    "nifty.config.cjs",
];

/// Walk upward from `start` and return the nearest existing Nifty config file.
pub fn find_config_file(start: &Path) -> Option<PathBuf> {
    let start = start.canonicalize().ok()?;
    let mut here = start.as_path();
    loop {
        for name in CONFIG_FILE_NAMES {
            let candidate = here.join(name);
            if candidate.is_file() {
                return candidate.canonicalize().ok();
            }
        }
        here = here.parent()?;
    }
}
