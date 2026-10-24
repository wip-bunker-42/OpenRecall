use crate::{AppError, AppResult, domain::settings::Settings};
use serde::{Deserialize, Serialize};
use std::path::Path;
use ts_rs::TS;
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Bootstrap {
    pub settings: Settings,
    pub schema_version: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    content = "payload",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum Request {
    Bootstrap {},
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
#[serde(tag = "kind", content = "payload", rename_all = "camelCase")]
pub enum Response {
    Bootstrap(Bootstrap),
}
pub fn export(output: &Path) -> AppResult<()> {
    let config = ts_rs::Config::default().with_out_dir(output);
    Request::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    Response::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    AppError::export_all(&config).map_err(|_| AppError::new("contractExport"))?;
    Ok(())
}
