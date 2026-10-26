use openrecall_core::AppResult;
use std::path::PathBuf;
#[derive(Clone)]
pub struct NativePaths {
    pub data: PathBuf,
    pub database: PathBuf,
    pub downloads: Option<PathBuf>,
}
impl NativePaths {
    pub fn from_roots(data: PathBuf, downloads: Option<PathBuf>) -> AppResult<Self> {
        std::fs::create_dir_all(&data)?;
        Ok(Self {
            database: data.join("openrecall.sqlite3"),
            data,
            downloads,
        })
    }
}
