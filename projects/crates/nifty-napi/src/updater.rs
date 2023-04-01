//! Node-API bindings for cargo + npm dependency updates.

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nifty_updater::{run_update, UpdateOptions as CoreUpdateOptions};

#[napi(object)]
pub struct UpdateRunOptions {
    pub cwd: Option<String>,
    pub interactive: Option<bool>,
}

fn map_err<T>(result: std::result::Result<T, String>) -> Result<T> {
    result.map_err(|message| Error::from_reason(message))
}

#[napi]
pub fn updater_run(options: UpdateRunOptions) -> Result<()> {
    map_err(run_update(CoreUpdateOptions {
        cwd: options.cwd.map(std::path::PathBuf::from),
        interactive: options.interactive.unwrap_or(false),
    }))
}
