use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct LibraryStoreData {
    libraries: Vec<Library>,
    active_library_id: Option<String>,
}

pub struct LibraryStore {
    file_path: PathBuf,
}

impl LibraryStore {
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            file_path: app_data_dir.join("libraries.json"),
        }
    }

    pub fn load(&self) -> Result<(Vec<Library>, Option<String>), AppError> {
        if !self.file_path.exists() {
            return Ok((Vec::new(), None));
        }
        let content = std::fs::read_to_string(&self.file_path)?;
        let data: LibraryStoreData = serde_json::from_str(&content)?;
        Ok((data.libraries, data.active_library_id))
    }

    fn save(&self, libraries: &[Library], active_id: Option<&str>) -> Result<(), AppError> {
        let data = LibraryStoreData {
            libraries: libraries.to_vec(),
            active_library_id: active_id.map(|s| s.to_string()),
        };
        let content = serde_json::to_string_pretty(&data)?;
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.file_path, content)?;
        Ok(())
    }

    pub fn add(&self, name: &str, path: &str) -> Result<Library, AppError> {
        let (mut libraries, active_id) = self.load()?;

        if libraries.iter().any(|lib| lib.path == path) {
            return Err(AppError::LibraryError(
                "同じパスのライブラリが既に存在します".to_string(),
            ));
        }

        let id = generate_id();
        let library = Library {
            id: id.clone(),
            name: name.to_string(),
            path: path.to_string(),
        };
        libraries.push(library.clone());

        let new_active = if active_id.is_some() {
            active_id.as_deref()
        } else {
            Some(id.as_str())
        };
        self.save(&libraries, new_active)?;
        Ok(library)
    }

    pub fn remove(&self, id: &str) -> Result<(), AppError> {
        let (mut libraries, active_id) = self.load()?;
        let original_len = libraries.len();
        libraries.retain(|lib| lib.id != id);
        if libraries.len() == original_len {
            return Err(AppError::LibraryError(
                "指定されたライブラリが見つかりません".to_string(),
            ));
        }

        let new_active = if active_id.as_deref() == Some(id) {
            libraries.first().map(|lib| lib.id.as_str())
        } else {
            active_id.as_deref()
        };
        self.save(&libraries, new_active)?;
        Ok(())
    }

    pub fn set_active(&self, id: &str) -> Result<(), AppError> {
        let (libraries, _) = self.load()?;
        if !libraries.iter().any(|lib| lib.id == id) {
            return Err(AppError::LibraryError(
                "指定されたライブラリが見つかりません".to_string(),
            ));
        }
        self.save(&libraries, Some(id))?;
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> Result<Option<Library>, AppError> {
        let (libraries, _) = self.load()?;
        Ok(libraries.into_iter().find(|lib| lib.id == id))
    }

    pub fn active_library(&self) -> Result<Option<Library>, AppError> {
        let (libraries, active_id) = self.load()?;
        let active_id = match active_id {
            Some(id) => id,
            None => return Ok(None),
        };
        Ok(libraries.into_iter().find(|lib| lib.id == active_id))
    }
}

fn generate_id() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let nanos = duration.as_nanos();
    format!("{:016x}", nanos & 0xFFFFFFFFFFFFFFFF)
}

#[cfg(test)]
#[path = "tests/library.rs"]
mod tests;
