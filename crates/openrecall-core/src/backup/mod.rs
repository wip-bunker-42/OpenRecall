pub(crate) mod files;
pub mod format;
mod validation;
#[derive(Clone, Copy)]
pub struct BackupLimits {
    pub max_bytes: u64,
}
impl Default for BackupLimits {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024,
        }
    }
}
pub(crate) struct PreparedRestore {
    pub preview: crate::contracts::backup::RestorePreview,
    pub envelope: format::BackupEnvelope,
}
