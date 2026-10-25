use super::{
    BackupLimits,
    format::{BackupEnvelope, FORMAT_VERSION, digest},
};
use crate::{AppError, AppResult};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};
pub(crate) fn read(path: &Path, limits: BackupLimits) -> AppResult<BackupEnvelope> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > limits.max_bytes {
        return Err(AppError::new("backupTooLarge"));
    }
    let mut bytes = Vec::new();
    file.take(limits.max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limits.max_bytes {
        return Err(AppError::new("backupTooLarge"));
    }
    let v: serde_json::Value = serde_json::from_slice(&bytes)?;
    if v["formatVersion"].as_u64() != Some(FORMAT_VERSION as u64) {
        return Err(AppError::new("unsupportedBackup"));
    }
    let envelope: BackupEnvelope = serde_json::from_value(v)?;
    if envelope.schema_version > crate::storage::SCHEMA_VERSION {
        return Err(AppError::new("unsupportedSchema"));
    }
    chrono::DateTime::parse_from_rfc3339(&envelope.exported_at)
        .map_err(|_| AppError::new("invalidBackup"))?;
    if envelope.payload_sha256 != digest(&envelope.data)? {
        return Err(AppError::new("backupDigest"));
    }
    super::validation::validate(&envelope.data)?;
    Ok(envelope)
}
pub(crate) fn write(
    destination: &Path,
    envelope: &BackupEnvelope,
    limits: BackupLimits,
) -> AppResult<(PathBuf, u64)> {
    // Hashing and writing use the same JSON number representation (including f32).
    let bytes = serde_json::to_vec_pretty(&serde_json::to_value(envelope)?)?;
    if bytes.len() as u64 > limits.max_bytes {
        return Err(AppError::new("backupTooLarge"));
    }
    let parent = destination
        .parent()
        .ok_or_else(|| AppError::new("fileAccess"))?;
    let mut temp = tempfile::NamedTempFile::new_in(parent)?;
    temp.write_all(&bytes)?;
    temp.as_file().sync_all()?;
    // Verify the exact bytes about to be finalized with the same parser/limits.
    read(temp.path(), limits)?;
    let stem = destination
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::new("fileAccess"))?;
    let extension = destination
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("json");
    for i in 0..10000 {
        let path = if i == 0 {
            destination.to_path_buf()
        } else {
            parent.join(format!("{stem}-{i}.{extension}"))
        };
        match temp.persist_noclobber(&path) {
            Ok(_) => return Ok((path, bytes.len() as u64)),
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => temp = e.file,
            Err(e) => return Err(e.error.into()),
        }
    }
    Err(AppError::new("fileCollision"))
}
