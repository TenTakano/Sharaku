use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;

const KNOWN_PLACEHOLDERS: &[&str] = &[
    "title", "artist", "year", "genre", "circle", "origin", "type",
];
const FORBIDDEN_CHARS: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

#[derive(Deserialize)]
pub struct WorkMetadata {
    pub title: String,
    pub artist: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub circle: Option<String>,
    pub origin: Option<String>,
    #[serde(default)]
    pub work_type: Option<String>,
}

pub fn validate_template(template: &str) -> Result<(), AppError> {
    if template.trim().is_empty() {
        return Err(AppError::InvalidTemplate(
            "テンプレートが空です".to_string(),
        ));
    }

    let mut has_title = false;
    let mut pos = 0;
    let bytes = template.as_bytes();

    while pos < bytes.len() {
        if bytes[pos] == b'{' {
            let close = template[pos..].find('}').map(|i| i + pos).ok_or_else(|| {
                AppError::InvalidTemplate("閉じられていないプレースホルダーがあります".into())
            })?;
            let name = &template[pos + 1..close];
            if name.is_empty() {
                return Err(AppError::InvalidTemplate(
                    "空のプレースホルダーがあります".into(),
                ));
            }
            if !KNOWN_PLACEHOLDERS.contains(&name) {
                return Err(AppError::InvalidTemplate(format!(
                    "未知のプレースホルダー: {{{}}}",
                    name
                )));
            }
            if name == "title" {
                has_title = true;
            }
            pos = close + 1;
        } else {
            pos += 1;
        }
    }

    if !has_title {
        return Err(AppError::InvalidTemplate("{title} は必須です".to_string()));
    }

    Ok(())
}

fn sanitize_segment(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .filter(|c| !FORBIDDEN_CHARS.contains(c))
        .collect::<String>()
        .trim()
        .to_string();
    if cleaned == ".." || cleaned == "." || cleaned.is_empty() {
        "_".to_string()
    } else {
        cleaned
    }
}

fn resolve_placeholder(name: &str, metadata: &WorkMetadata) -> String {
    match name {
        "title" => metadata.title.clone(),
        "artist" => metadata
            .artist
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        "year" => metadata
            .year
            .map(|y| y.to_string())
            .unwrap_or_else(|| "Unknown".to_string()),
        "genre" => metadata
            .genre
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        "circle" => metadata
            .circle
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        "origin" => metadata
            .origin
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        "type" => metadata
            .work_type
            .clone()
            .unwrap_or_else(|| "Unknown".to_string()),
        _ => "Unknown".to_string(),
    }
}

pub fn render_template(template: &str, metadata: &WorkMetadata) -> String {
    let segments: Vec<&str> = template.split('/').collect();
    segments
        .iter()
        .map(|segment| {
            let mut result = String::new();
            let mut chars = segment.char_indices().peekable();
            while let Some((i, ch)) = chars.next() {
                if ch == '{' {
                    if let Some(close_offset) = segment[i..].find('}') {
                        let name = &segment[i + 1..i + close_offset];
                        result.push_str(&resolve_placeholder(name, metadata));
                        while let Some(&(j, _)) = chars.peek() {
                            if j <= i + close_offset {
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    } else {
                        result.push('{');
                    }
                } else {
                    result.push(ch);
                }
            }
            sanitize_segment(&result)
        })
        .collect::<Vec<_>>()
        .join("/")
}

pub fn resolve_work_path(library_root: &Path, template: &str, metadata: &WorkMetadata) -> PathBuf {
    let rendered = render_template(template, metadata);
    let resolved = library_root.join(&rendered);
    let normalized = normalize_path(&resolved);
    let root_normalized = normalize_path(library_root);
    if !normalized.starts_with(&root_normalized) {
        library_root.join("_invalid_path")
    } else {
        normalized
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

pub fn resolve_unique_work_path(
    library_root: &Path,
    template: &str,
    metadata: &WorkMetadata,
) -> PathBuf {
    let base = resolve_work_path(library_root, template, metadata);
    if !base.exists() {
        return base;
    }
    let suffix: u16 = rand_suffix();
    let dir_name = format!(
        "{}_{:04x}",
        base.file_name().unwrap().to_string_lossy(),
        suffix
    );
    base.with_file_name(dir_name)
}

fn rand_suffix() -> u16 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let mut hasher = DefaultHasher::new();
    SystemTime::now().hash(&mut hasher);
    std::thread::current().id().hash(&mut hasher);
    hasher.finish() as u16
}

pub fn sample_metadata() -> WorkMetadata {
    WorkMetadata {
        title: "My Artwork".to_string(),
        artist: Some("Artist Name".to_string()),
        year: Some(2025),
        genre: Some("Illustration".to_string()),
        circle: Some("Circle".to_string()),
        origin: Some("Original".to_string()),
        work_type: None,
    }
}

#[cfg(test)]
#[path = "tests/template.rs"]
mod tests;
