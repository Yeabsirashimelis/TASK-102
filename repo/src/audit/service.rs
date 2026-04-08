use diesel::prelude::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::audit_log::NewAuditEntry;
use crate::schema::audit_log;

/// Record an audit entry for a write operation.
pub fn record(
    conn: &mut PgConnection,
    user_id: Option<Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    http_method: &str,
    http_path: &str,
    before_state: Option<&[u8]>,
    after_state: Option<&[u8]>,
    metadata: Option<serde_json::Value>,
    ip_address: Option<String>,
) -> Result<(), AppError> {
    let entry = NewAuditEntry {
        user_id,
        action: action.to_string(),
        resource_type: resource_type.to_string(),
        resource_id,
        http_method: http_method.to_string(),
        http_path: http_path.to_string(),
        before_hash: before_state.map(|d| hash_sha256(d)),
        after_hash: after_state.map(|d| hash_sha256(d)),
        metadata,
        ip_address,
    };

    diesel::insert_into(audit_log::table)
        .values(&entry)
        .execute(conn)?;

    Ok(())
}

/// Convenience wrapper for handler-level audit with before/after state.
/// Accepts serializable before/after values, canonically serializes them,
/// and records the audit entry. Sensitive fields must be excluded by the
/// caller before passing values here.
pub fn audit_write(
    conn: &mut PgConnection,
    user_id: Uuid,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    before: Option<&serde_json::Value>,
    after: Option<&serde_json::Value>,
) -> Result<(), AppError> {
    let before_bytes = before.map(|v| canonical_bytes(v));
    let after_bytes = after.map(|v| canonical_bytes(v));

    record(
        conn,
        Some(user_id),
        action,
        resource_type,
        resource_id,
        match action {
            "create" => "POST",
            "update" => "PUT",
            "delete" => "DELETE",
            _ => "POST",
        },
        &format!("/api/v1/{}", resource_type),
        before_bytes.as_deref(),
        after_bytes.as_deref(),
        None,
        None,
    )
}

/// Deterministic canonical serialization for hashing.
/// Uses serde_json with sorted keys (default behavior).
fn canonical_bytes(value: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(value).unwrap_or_default()
}

/// Compute SHA-256 hash of arbitrary bytes, returned as hex string.
pub fn hash_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let h1 = hash_sha256(b"hello");
        let h2 = hash_sha256(b"hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_different_inputs() {
        assert_ne!(hash_sha256(b"hello"), hash_sha256(b"world"));
    }

    #[test]
    fn test_hash_length() {
        assert_eq!(hash_sha256(b"test").len(), 64);
    }

    #[test]
    fn test_hash_empty_input() {
        assert_eq!(
            hash_sha256(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_hash_known_value() {
        assert_eq!(
            hash_sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_canonical_bytes_deterministic() {
        let v = serde_json::json!({"b": 2, "a": 1});
        let b1 = canonical_bytes(&v);
        let b2 = canonical_bytes(&v);
        assert_eq!(b1, b2);
    }
}
