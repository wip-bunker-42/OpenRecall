use super::format::{BackupData, FORMAT_VERSION, digest_value};
use crate::{AppError, AppResult};
use serde::Deserialize;

/// An authenticated source envelope with its payload upgraded for this application.
/// It is deliberately not serializable: original metadata describes the source bytes.
#[derive(Clone, Debug)]
pub struct ValidatedBackup {
    pub format_version: u32,
    pub schema_version: u32,
    pub app_version: String,
    pub exported_at: String,
    pub payload_sha256: String,
    pub data: BackupData,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SourceEnvelope {
    format_version: u32,
    schema_version: u32,
    app_version: String,
    exported_at: String,
    payload_sha256: String,
    data: serde_json::Value,
}

pub fn decode(bytes: &[u8]) -> AppResult<ValidatedBackup> {
    let raw: serde_json::Value = serde_json::from_slice(bytes)?;
    if !matches!(raw["formatVersion"].as_u64(), Some(1) | Some(2)) {
        return Err(AppError::new("unsupportedBackup"));
    }
    let source: SourceEnvelope = serde_json::from_value(raw)?;
    if source.schema_version > crate::SCHEMA_VERSION {
        return Err(AppError::new("unsupportedSchema"));
    }
    chrono::DateTime::parse_from_rfc3339(&source.exported_at)
        .map_err(|_| AppError::new("invalidBackup"))?;
    // Authenticate original JSON before any serde defaults, upgrades or field removal.
    if source.payload_sha256 != digest_value(&source.data)? {
        return Err(AppError::new("backupDigest"));
    }
    let data = match source.format_version {
        1 if source.schema_version == 8 => super::legacy_v1::upgrade(source.data)?,
        FORMAT_VERSION => serde_json::from_value(source.data)?,
        _ => return Err(AppError::new("unsupportedSchema")),
    };
    super::validation::validate(&data)?;
    Ok(ValidatedBackup {
        format_version: source.format_version,
        schema_version: source.schema_version,
        app_version: source.app_version,
        exported_at: source.exported_at,
        payload_sha256: source.payload_sha256,
        data,
    })
}
