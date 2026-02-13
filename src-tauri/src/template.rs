use std::path::{Path, PathBuf};

use crate::error::AppError;

const KNOWN_PLACEHOLDERS: &[&str] = &["title", "artist", "year", "genre", "circle", "origin"];
const FORBIDDEN_CHARS: &[char] = &['\\', ':', '*', '?', '"', '<', '>', '|'];

pub struct WorkMetadata {
    pub title: String,
    pub artist: Option<String>,
    pub year: Option<i32>,
    pub genre: Option<String>,
    pub circle: Option<String>,
    pub origin: Option<String>,
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
            let close = template[pos..]
                .find('}')
                .map(|i| i + pos)
                .ok_or_else(|| {
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
        return Err(AppError::InvalidTemplate(
            "{title} は必須です".to_string(),
        ));
    }

    Ok(())
}

fn sanitize_segment(s: &str) -> String {
    s.chars()
        .filter(|c| !FORBIDDEN_CHARS.contains(c))
        .collect::<String>()
        .trim()
        .to_string()
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
        _ => "Unknown".to_string(),
    }
}

pub fn render_template(template: &str, metadata: &WorkMetadata) -> String {
    let segments: Vec<&str> = template.split('/').collect();
    segments
        .iter()
        .map(|segment| {
            let mut result = String::new();
            let mut pos = 0;
            let bytes = segment.as_bytes();
            while pos < bytes.len() {
                if bytes[pos] == b'{' {
                    if let Some(close_offset) = segment[pos..].find('}') {
                        let close = pos + close_offset;
                        let name = &segment[pos + 1..close];
                        result.push_str(&resolve_placeholder(name, metadata));
                        pos = close + 1;
                    } else {
                        result.push('{');
                        pos += 1;
                    }
                } else {
                    result.push(bytes[pos] as char);
                    pos += 1;
                }
            }
            sanitize_segment(&result)
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[allow(dead_code)]
pub fn resolve_work_path(
    library_root: &Path,
    template: &str,
    metadata: &WorkMetadata,
) -> PathBuf {
    let rendered = render_template(template, metadata);
    library_root.join(rendered)
}

#[allow(dead_code)]
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
    }
}

#[cfg(test)]
#[path = "tests/template.rs"]
mod tests;
