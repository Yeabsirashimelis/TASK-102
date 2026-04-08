use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::file_attachment::{ALLOWED_CONTENT_TYPES, MAX_FILE_SIZE};

/// Base directory for file storage. Configurable via FILE_STORAGE_PATH env var.
pub fn storage_base() -> PathBuf {
    let base = std::env::var("FILE_STORAGE_PATH").unwrap_or_else(|_| "./data/uploads".into());
    PathBuf::from(base)
}

/// Validates the content type is allowed.
pub fn validate_content_type(content_type: &str) -> Result<(), AppError> {
    if !ALLOWED_CONTENT_TYPES.contains(&content_type) {
        return Err(AppError::Validation(format!(
            "File type '{}' is not allowed. Allowed types: PDF, JPG, PNG, CSV, XLSX",
            content_type
        )));
    }
    Ok(())
}

/// Validates the file size is within the allowed limit.
pub fn validate_file_size(size: u64) -> Result<(), AppError> {
    if size > MAX_FILE_SIZE {
        return Err(AppError::Validation(format!(
            "File size {} bytes exceeds maximum of {} bytes (10 MB)",
            size, MAX_FILE_SIZE
        )));
    }
    Ok(())
}

/// Infer content type from file extension.
pub fn content_type_from_filename(filename: &str) -> Result<String, AppError> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "pdf" => Ok("application/pdf".into()),
        "jpg" | "jpeg" => Ok("image/jpeg".into()),
        "png" => Ok("image/png".into()),
        "csv" => Ok("text/csv".into()),
        "xlsx" => Ok(
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".into(),
        ),
        _ => Err(AppError::Validation(format!(
            "File extension '.{}' is not allowed. Allowed: pdf, jpg, png, csv, xlsx",
            ext
        ))),
    }
}

/// Saves file bytes to disk and returns (disk_path, sha256_hex).
/// Files are organized as: {base}/{participant_id}/{uuid}.{ext}
pub fn save_file(
    participant_id: Uuid,
    original_filename: &str,
    data: &[u8],
) -> Result<(String, String), AppError> {
    let ext = Path::new(original_filename)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin");

    let file_id = Uuid::new_v4();
    let dir = storage_base().join(participant_id.to_string());

    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("Failed to create storage dir: {}", e)))?;

    let file_name = format!("{}.{}", file_id, ext);
    let file_path = dir.join(&file_name);

    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| AppError::Internal(format!("Failed to create file: {}", e)))?;
    file.write_all(data)
        .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

    // Compute SHA-256
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = format!("{:x}", hasher.finalize());

    let disk_path = file_path
        .to_str()
        .unwrap_or("")
        .to_string();

    Ok((disk_path, hash))
}

/// Reads a file from disk by its stored path.
pub fn read_file(path: &str) -> Result<Vec<u8>, AppError> {
    std::fs::read(path)
        .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))
}

/// Deletes a file from disk.
pub fn delete_file(path: &str) -> Result<(), AppError> {
    if Path::new(path).exists() {
        std::fs::remove_file(path)
            .map_err(|e| AppError::Internal(format!("Failed to delete file: {}", e)))?;
    }
    Ok(())
}

/// Validates that a path is safe (no traversal, stays under storage base).
/// Returns Err if the path contains "..", is absolute, or escapes the base.
pub fn validate_safe_path(user_path: &str) -> Result<(), AppError> {
    if user_path.contains("..") || user_path.starts_with('/') || user_path.starts_with('\\') {
        return Err(AppError::Validation(
            "Path traversal or absolute paths are not allowed".into(),
        ));
    }
    // Check for null bytes
    if user_path.contains('\0') {
        return Err(AppError::Validation("Invalid path characters".into()));
    }
    Ok(())
}

/// Save an artifact (receipt/export) to managed storage under a category subdir.
/// Returns (server_managed_path, sha256_hex).
/// Never trusts caller-provided paths — always generates paths internally.
pub fn save_artifact(
    category: &str,
    entity_id: Uuid,
    extension: &str,
    data: &[u8],
) -> Result<(String, String), AppError> {
    validate_file_size(data.len() as u64)?;

    let file_id = Uuid::new_v4();
    let dir = storage_base().join(category).join(entity_id.to_string());

    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Internal(format!("Failed to create storage dir: {}", e)))?;

    let safe_ext = extension
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>();
    let file_name = format!("{}.{}", file_id, safe_ext);
    let file_path = dir.join(&file_name);

    let mut file = std::fs::File::create(&file_path)
        .map_err(|e| AppError::Internal(format!("Failed to create file: {}", e)))?;
    std::io::Write::write_all(&mut file, data)
        .map_err(|e| AppError::Internal(format!("Failed to write file: {}", e)))?;

    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = format!("{:x}", hasher.finalize());

    let disk_path = file_path.to_str().unwrap_or("").to_string();
    Ok((disk_path, hash))
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- content_type_from_filename ---

    #[test]
    fn test_content_type_pdf() {
        assert_eq!(content_type_from_filename("doc.pdf").unwrap(), "application/pdf");
    }

    #[test]
    fn test_content_type_jpg() {
        assert_eq!(content_type_from_filename("photo.jpg").unwrap(), "image/jpeg");
    }

    #[test]
    fn test_content_type_jpeg() {
        assert_eq!(content_type_from_filename("photo.jpeg").unwrap(), "image/jpeg");
    }

    #[test]
    fn test_content_type_png() {
        assert_eq!(content_type_from_filename("image.png").unwrap(), "image/png");
    }

    #[test]
    fn test_content_type_csv() {
        assert_eq!(content_type_from_filename("data.csv").unwrap(), "text/csv");
    }

    #[test]
    fn test_content_type_xlsx() {
        let ct = content_type_from_filename("report.xlsx").unwrap();
        assert_eq!(ct, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    }

    #[test]
    fn test_content_type_case_insensitive() {
        assert_eq!(content_type_from_filename("DOC.PDF").unwrap(), "application/pdf");
        assert_eq!(content_type_from_filename("image.PNG").unwrap(), "image/png");
    }

    #[test]
    fn test_content_type_disallowed_exe() {
        assert!(content_type_from_filename("virus.exe").is_err());
    }

    #[test]
    fn test_content_type_disallowed_txt() {
        assert!(content_type_from_filename("notes.txt").is_err());
    }

    #[test]
    fn test_content_type_no_extension() {
        assert!(content_type_from_filename("noext").is_err());
    }

    // --- validate_content_type ---

    #[test]
    fn test_validate_allowed_types() {
        for ct in ALLOWED_CONTENT_TYPES {
            assert!(validate_content_type(ct).is_ok(), "Should allow {}", ct);
        }
    }

    #[test]
    fn test_validate_disallowed_type() {
        assert!(validate_content_type("application/javascript").is_err());
        assert!(validate_content_type("text/html").is_err());
    }

    // --- validate_file_size ---

    #[test]
    fn test_size_within_limit() {
        assert!(validate_file_size(0).is_ok());
        assert!(validate_file_size(1024).is_ok());
        assert!(validate_file_size(MAX_FILE_SIZE).is_ok());
    }

    #[test]
    fn test_size_over_limit() {
        assert!(validate_file_size(MAX_FILE_SIZE + 1).is_err());
    }

    // --- save_file / read_file / delete_file roundtrip ---

    // --- validate_safe_path ---

    #[test]
    fn test_safe_path_traversal_rejected() {
        assert!(validate_safe_path("../etc/passwd").is_err());
        assert!(validate_safe_path("foo/../../bar").is_err());
    }

    #[test]
    fn test_safe_path_absolute_rejected() {
        assert!(validate_safe_path("/etc/passwd").is_err());
        assert!(validate_safe_path("\\windows\\system32").is_err());
    }

    #[test]
    fn test_safe_path_null_byte_rejected() {
        assert!(validate_safe_path("file\0.txt").is_err());
    }

    #[test]
    fn test_safe_path_valid() {
        assert!(validate_safe_path("data/file.csv").is_ok());
        assert!(validate_safe_path("exports/report.xlsx").is_ok());
    }

    #[test]
    fn test_save_read_delete_roundtrip() {
        std::env::set_var("FILE_STORAGE_PATH", std::env::temp_dir().join("retailops_test_storage").to_str().unwrap());
        let pid = Uuid::new_v4();
        let data = b"test file contents";
        let (path, hash) = save_file(pid, "test.pdf", data).unwrap();

        // Hash should be deterministic
        assert_eq!(hash.len(), 64); // SHA-256 hex
        let (_, hash2) = save_file(pid, "test2.pdf", data).unwrap();
        assert_eq!(hash, hash2); // same content = same hash

        // Read back
        let read_data = read_file(&path).unwrap();
        assert_eq!(read_data, data);

        // Delete
        assert!(delete_file(&path).is_ok());
        assert!(read_file(&path).is_err());

        // Delete non-existent is ok
        assert!(delete_file(&path).is_ok());
    }
}
