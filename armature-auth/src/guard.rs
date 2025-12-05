// Authentication guards

use crate::{AuthError, AuthUser, Result};
use armature_core::HttpRequest;
use async_trait::async_trait;

/// Guard trait for protecting routes
#[async_trait]
pub trait Guard: Send + Sync {
    /// Check if the request can proceed
    async fn can_activate(&self, request: &HttpRequest) -> Result<bool>;
}

/// Authentication guard - ensures user is authenticated
#[derive(Clone)]
pub struct AuthGuard;

impl AuthGuard {
    pub fn new() -> Self {
        Self
    }

    /// Extract user from request
    pub fn extract_user<T: AuthUser>(&self, _request: &HttpRequest) -> Result<T> {
        // In a real implementation, this would:
        // 1. Extract JWT token from Authorization header
        // 2. Verify the token
        // 3. Extract user info from claims
        // 4. Load user from database if needed

        // For now, return error - this should be implemented by the application
        Err(AuthError::Unauthorized)
    }
}

impl Default for AuthGuard {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Guard for AuthGuard {
    async fn can_activate(&self, request: &HttpRequest) -> Result<bool> {
        // Check for Authorization header
        let auth_header = request
            .headers
            .get("authorization")
            .ok_or(AuthError::Unauthorized)?;

        // Check for Bearer token
        if !auth_header.starts_with("Bearer ") {
            return Err(AuthError::InvalidToken(
                "Invalid authorization format".to_string(),
            ));
        }

        // Token exists - authentication check passed
        // In production, you'd verify the token here
        Ok(true)
    }
}

/// Role-based authorization guard
#[derive(Clone)]
pub struct RoleGuard {
    required_roles: Vec<String>,
    require_all: bool,
}

impl RoleGuard {
    /// Create a guard that requires ANY of the roles
    pub fn any(roles: Vec<String>) -> Self {
        Self {
            required_roles: roles,
            require_all: false,
        }
    }

    /// Create a guard that requires ALL of the roles
    pub fn all(roles: Vec<String>) -> Self {
        Self {
            required_roles: roles,
            require_all: true,
        }
    }

    /// Check if user has required roles
    pub fn check_roles<T: AuthUser>(&self, user: &T) -> bool {
        let role_refs: Vec<&str> = self.required_roles.iter().map(|s| s.as_str()).collect();

        if self.require_all {
            user.has_all_roles(&role_refs)
        } else {
            user.has_any_role(&role_refs)
        }
    }
}

#[async_trait]
impl Guard for RoleGuard {
    async fn can_activate(&self, request: &HttpRequest) -> Result<bool> {
        // First check authentication
        let auth_guard = AuthGuard::new();
        auth_guard.can_activate(request).await?;

        // Then check roles
        // In production, extract user from request and check roles
        // For now, this is a placeholder

        Ok(true)
    }
}

/// Permission-based authorization guard
#[derive(Clone)]
pub struct PermissionGuard {
    required_permissions: Vec<String>,
    require_all: bool,
}

impl PermissionGuard {
    /// Create a guard that requires ANY of the permissions
    pub fn any(permissions: Vec<String>) -> Self {
        Self {
            required_permissions: permissions,
            require_all: false,
        }
    }

    /// Create a guard that requires ALL of the permissions
    pub fn all(permissions: Vec<String>) -> Self {
        Self {
            required_permissions: permissions,
            require_all: true,
        }
    }

    /// Check if user has required permissions
    pub fn check_permissions<T: AuthUser>(&self, user: &T) -> bool {
        if self.require_all {
            self.required_permissions
                .iter()
                .all(|perm| user.has_permission(perm))
        } else {
            self.required_permissions
                .iter()
                .any(|perm| user.has_permission(perm))
        }
    }
}

#[async_trait]
impl Guard for PermissionGuard {
    async fn can_activate(&self, request: &HttpRequest) -> Result<bool> {
        // First check authentication
        let auth_guard = AuthGuard::new();
        auth_guard.can_activate(request).await?;

        // Then check permissions
        // In production, extract user from request and check permissions

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UserContext;

    #[test]
    fn test_role_guard() {
        let user = UserContext::new("user123".to_string())
            .with_roles(vec!["admin".to_string(), "user".to_string()]);

        // Test ANY
        let guard = RoleGuard::any(vec!["admin".to_string()]);
        assert!(guard.check_roles(&user));

        let guard = RoleGuard::any(vec!["guest".to_string()]);
        assert!(!guard.check_roles(&user));

        // Test ALL
        let guard = RoleGuard::all(vec!["admin".to_string(), "user".to_string()]);
        assert!(guard.check_roles(&user));

        let guard = RoleGuard::all(vec!["admin".to_string(), "guest".to_string()]);
        assert!(!guard.check_roles(&user));
    }

    #[test]
    fn test_permission_guard() {
        let user = UserContext::new("user123".to_string())
            .with_permissions(vec!["read".to_string(), "write".to_string()]);

        // Test ANY
        let guard = PermissionGuard::any(vec!["read".to_string()]);
        assert!(guard.check_permissions(&user));

        let guard = PermissionGuard::any(vec!["delete".to_string()]);
        assert!(!guard.check_permissions(&user));

        // Test ALL
        let guard = PermissionGuard::all(vec!["read".to_string(), "write".to_string()]);
        assert!(guard.check_permissions(&user));

        let guard = PermissionGuard::all(vec!["read".to_string(), "delete".to_string()]);
        assert!(!guard.check_permissions(&user));
    }
}
