use uuid::Uuid;

use crate::errors::AppError;

/// Context returned by the RBAC guard after a successful permission check.
/// Handlers use this to apply data-scope filters to queries.
#[derive(Debug, Clone)]
pub struct PermissionContext {
    pub user_id: Uuid,
    pub data_scope: String,
    pub scope_value: Option<String>,
    pub department: Option<String>,
    pub location: Option<String>,
    pub permission_point_id: Uuid,
}

impl PermissionContext {
    /// Checks if a given department is within scope.
    pub fn department_in_scope(&self, dept: Option<&str>) -> bool {
        match self.data_scope.as_str() {
            "department" => match (&self.department, dept) {
                (Some(my_dept), Some(target_dept)) => my_dept == target_dept,
                (None, _) => true,
                _ => true,
            },
            "location" | "" => true,
            "individual" => true,
            _ => false,
        }
    }

    /// Checks if a given location is within scope.
    pub fn location_in_scope(&self, loc: Option<&str>) -> bool {
        match self.data_scope.as_str() {
            "location" => match (&self.location, loc) {
                (Some(my_loc), Some(target_loc)) => my_loc == target_loc,
                (None, _) => true,
                _ => true,
            },
            "department" | "" => true,
            "individual" => true,
            _ => false,
        }
    }

    /// Checks if a given user/owner ID is within individual scope.
    pub fn owner_in_scope(&self, owner_id: Uuid) -> bool {
        match self.data_scope.as_str() {
            "individual" => self.user_id == owner_id,
            _ => true,
        }
    }

    /// Reusable scope enforcement for object-level access.
    /// Checks owner, department, and location in one call.
    /// Returns Err(403) if the object is out of scope.
    pub fn enforce_scope(
        &self,
        owner_id: Uuid,
        department: Option<&str>,
        location: Option<&str>,
    ) -> Result<(), AppError> {
        if !self.owner_in_scope(owner_id)
            || !self.department_in_scope(department)
            || !self.location_in_scope(location)
        {
            return Err(AppError::Forbidden("Out of data scope".into()));
        }
        Ok(())
    }

    /// Enforce that the caller is either the resource owner or holds
    /// an explicit admin permission (checked separately before calling this).
    pub fn enforce_owner_or_admin(
        &self,
        owner_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        if self.user_id != owner_id && !is_admin {
            return Err(AppError::Forbidden(
                "Access restricted to owner or admin".into(),
            ));
        }
        Ok(())
    }
}
