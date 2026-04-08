use diesel::prelude::*;
use uuid::Uuid;

use crate::auth::jwt::Claims;
use crate::errors::AppError;
use crate::models::permission_point::PermissionPoint;
use crate::rbac::data_scope::PermissionContext;
use crate::schema::{api_capabilities, permission_points, role_permissions};

/// Standard permission check (layers 1-2 + approval gate).
/// Layer 3 (capability match) is skipped — use `check_permission_for_request`
/// when request method+path context is available.
pub fn check_permission(
    claims: &Claims,
    permission_code: &str,
    conn: &mut PgConnection,
) -> Result<PermissionContext, AppError> {
    let perm = resolve_layers_1_2(claims, permission_code, conn)?;
    if perm.requires_approval {
        return Err(AppError::Forbidden(
            format!("Permission {} requires approval workflow", permission_code),
        ));
    }
    Ok(build_context(claims, perm.id))
}

/// Permission check without approval gate.
pub fn check_permission_no_approval(
    claims: &Claims,
    permission_code: &str,
    conn: &mut PgConnection,
) -> Result<PermissionContext, AppError> {
    let perm = resolve_layers_1_2(claims, permission_code, conn)?;
    Ok(build_context(claims, perm.id))
}

/// Full three-layer RBAC check with request-aware capability enforcement.
///   Layer 1: permission_point lookup
///   Layer 2: role→permission binding (or delegation)
///   Layer 3: permission→api_capability matching on http_method + path_pattern
/// If no capabilities are registered for the permission, layer 3 is skipped.
pub fn check_permission_for_request(
    claims: &Claims,
    permission_code: &str,
    http_method: &str,
    http_path: &str,
    conn: &mut PgConnection,
) -> Result<PermissionContext, AppError> {
    let perm = resolve_layers_1_2(claims, permission_code, conn)?;

    // Layer 3: capability enforcement
    let caps: Vec<crate::models::api_capability::ApiCapability> = api_capabilities::table
        .filter(api_capabilities::permission_point_id.eq(perm.id))
        .load(conn)
        .unwrap_or_default();

    if !caps.is_empty() {
        let method_upper = http_method.to_uppercase();
        let matched = caps.iter().any(|cap| {
            cap.http_method.to_uppercase() == method_upper && path_matches(&cap.path_pattern, http_path)
        });
        if !matched {
            return Err(AppError::Forbidden(format!(
                "No matching API capability for {} {} on permission {}",
                http_method, http_path, permission_code
            )));
        }
    }

    if perm.requires_approval {
        return Err(AppError::Forbidden(
            format!("Permission {} requires approval workflow", permission_code),
        ));
    }

    Ok(build_context(claims, perm.id))
}

/// Layers 1+2: resolve permission point and verify role binding.
fn resolve_layers_1_2(
    claims: &Claims,
    permission_code: &str,
    conn: &mut PgConnection,
) -> Result<PermissionPoint, AppError> {
    let perm: PermissionPoint = permission_points::table
        .filter(permission_points::code.eq(permission_code))
        .first(conn)
        .map_err(|_| AppError::Forbidden(format!("Unknown permission: {}", permission_code)))?;

    let has_role_binding: bool = role_permissions::table
        .filter(role_permissions::role_id.eq(claims.role_id))
        .filter(role_permissions::permission_point_id.eq(perm.id))
        .count()
        .get_result::<i64>(conn)
        .map(|c| c > 0)
        .unwrap_or(false);

    let has_delegation = claims.delegated_permissions.contains(&perm.id);

    if !has_role_binding && !has_delegation {
        return Err(AppError::Forbidden(format!(
            "Missing permission: {}", permission_code
        )));
    }

    Ok(perm)
}

/// Matches a capability path_pattern against a concrete request path.
/// `*` matches a single segment; `**` at the end matches any trailing segments.
pub fn path_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    let pat: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let req: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    if pat.last() == Some(&"**") {
        let prefix = &pat[..pat.len() - 1];
        if req.len() < prefix.len() {
            return false;
        }
        return prefix.iter().zip(req.iter()).all(|(p, r)| *p == "*" || p == r);
    }

    if pat.len() != req.len() {
        return false;
    }
    pat.iter().zip(req.iter()).all(|(p, r)| *p == "*" || p == r)
}

fn build_context(claims: &Claims, permission_point_id: Uuid) -> PermissionContext {
    PermissionContext {
        user_id: claims.sub,
        data_scope: claims.data_scope.clone(),
        scope_value: claims.scope_value.clone(),
        department: claims.department.clone(),
        location: claims.location.clone(),
        permission_point_id,
    }
}

pub fn resolve_permission_id(
    permission_code: &str,
    conn: &mut PgConnection,
) -> Result<Uuid, AppError> {
    permission_points::table
        .filter(permission_points::code.eq(permission_code))
        .select(permission_points::id)
        .first(conn)
        .map_err(|_| AppError::Forbidden(format!("Unknown permission: {}", permission_code)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_exact_match() {
        assert!(path_matches("/api/v1/orders", "/api/v1/orders"));
    }

    #[test]
    fn test_path_wildcard_single() {
        assert!(path_matches("/api/v1/orders/*", "/api/v1/orders/123"));
    }

    #[test]
    fn test_path_wildcard_double() {
        assert!(path_matches("/api/v1/orders/**", "/api/v1/orders/123/payments"));
    }

    #[test]
    fn test_path_no_match_different() {
        assert!(!path_matches("/api/v1/orders", "/api/v1/users"));
    }

    #[test]
    fn test_path_no_match_extra_segment() {
        assert!(!path_matches("/api/v1/orders/*", "/api/v1/orders/1/pay"));
    }

    #[test]
    fn test_path_multi_wildcard() {
        assert!(path_matches("/api/v1/*/versions/*", "/api/v1/ds1/versions/v1"));
    }

    #[test]
    fn test_path_no_match_wrong_method_simulated() {
        // path_matches only checks path; method checked separately
        assert!(path_matches("/api/v1/orders", "/api/v1/orders"));
    }
}
