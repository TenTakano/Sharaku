use std::path::Path;

use crate::db;
use crate::importer;

pub fn parse_view_uri(uri: &str) -> Option<(i64, usize)> {
    let idx = uri.find("view/")?;
    let rest = &uri[idx + 5..];
    let rest = rest.split('?').next()?;
    let rest = rest.split('#').next()?;
    let mut parts = rest.splitn(2, '/');
    let work_id: i64 = parts.next()?.parse().ok()?;
    let page_index: usize = parts.next()?.parse().ok()?;
    Some((work_id, page_index))
}

pub fn handle_view_request(
    app_data_dir: &Path,
    work_id: i64,
    page_index: usize,
) -> tauri::http::Response<Vec<u8>> {
    match load_image(app_data_dir, work_id, page_index) {
        Ok((data, content_type)) => tauri::http::Response::builder()
            .status(200)
            .header("Content-Type", content_type)
            .body(data)
            .unwrap(),
        Err(status) => tauri::http::Response::builder()
            .status(status)
            .body(Vec::new())
            .unwrap(),
    }
}

fn load_image(
    app_data_dir: &Path,
    work_id: i64,
    page_index: usize,
) -> Result<(Vec<u8>, &'static str), u16> {
    let conn = db::open_db(app_data_dir).map_err(|_| 500u16)?;
    let work = db::get_work(&conn, work_id).map_err(|_| 404u16)?;

    if work.work_type == "folder" {
        let images = importer::list_images_in_folder(Path::new(&work.path)).map_err(|e| {
            if let crate::error::AppError::Io(ref io_err) = e {
                if io_err.kind() == std::io::ErrorKind::NotFound {
                    return 404u16;
                }
            }
            500u16
        })?;
        let file_path = images.get(page_index).ok_or(404u16)?;
        let data = std::fs::read(file_path).map_err(|_| 404u16)?;
        let content_type = content_type_from_path(&file_path.to_string_lossy());
        Ok((data, content_type))
    } else {
        if page_index != 0 {
            return Err(404);
        }
        let data = std::fs::read(&work.path).map_err(|_| 404u16)?;
        let content_type = content_type_from_path(&work.path);
        Ok((data, content_type))
    }
}

fn content_type_from_path(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
#[path = "tests/viewer.rs"]
mod tests;
