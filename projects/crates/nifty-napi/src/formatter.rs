//! Node-API bindings for workspace formatting (`oxc_formatter` + `cargo fmt`).

use napi::bindgen_prelude::*;
use napi_derive::napi;
use nifty_formatter::{run_format, FormatReport as CoreFormatReport, RunFormatOptions};

#[napi(object)]
pub struct FormatRunOptions {
    pub cwd: Option<String>,
    pub check: Option<bool>,
    pub includes: Option<Vec<String>>,
    pub excludes: Option<Vec<String>>,
    pub rust: Option<bool>,
    pub javascript: Option<bool>,
    pub style_config: Option<String>,
}

#[napi(object)]
pub struct FormatReportNapi {
    pub formatted: u32,
    pub unchanged: u32,
    pub errors: Vec<String>,
}

fn map_err<T>(result: std::result::Result<T, String>) -> Result<T> {
    result.map_err(|message| Error::from_reason(message))
}

#[napi]
pub fn formatter_run(options: FormatRunOptions) -> Result<FormatReportNapi> {
    let report = map_err(run_format(RunFormatOptions {
        cwd: options.cwd.map(std::path::PathBuf::from),
        check: options.check.unwrap_or(false),
        includes: options.includes,
        excludes: options.excludes,
        rust: options.rust,
        javascript: options.javascript,
        style_config: options.style_config.map(std::path::PathBuf::from),
    }))?;
    Ok(to_napi_report(report))
}

fn to_napi_report(report: CoreFormatReport) -> FormatReportNapi {
    FormatReportNapi {
        formatted: report.formatted as u32,
        unchanged: report.unchanged as u32,
        errors: report.errors,
    }
}
