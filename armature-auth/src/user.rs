// User traits and context

use serde::{Deserialize, Serialize};

/// Trait for authenticated users
pub trait AuthUser: Send + Sync {
    /// Get user ID
    fn user_id(&self) -> String;

    /// Check if user is active
    fn is_active(&self) -> bool {
        true
    }

    /// Check if user has a role
    fn has_role(&self, role: &str) -> bool;

    /// Check if user has any of the roles
    fn has_any_role(&self, roles: &[&str]) -> bool {
        roles.iter().any(|role| self.has_role(role))
    }

    /// Check if user has all roles
    fn has_all_roles(&self, roles: &[&str]) -> bool {
        roles.iter().all(|role| self.has_role(role))
    }

    /// Check if user has a permission
    fn has_permission(&self, permission: &str) -> bool;
}

/// User context extracted from requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID
    pub user_id: String,

    /// User email
    pub email: Option<String>,

    /// User roles
    pub roles: Vec<String>,

    /// User permissions
    pub permissions: Vec<String>,

    /// Custom metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl UserContext {
    /// Create a new user context
    pub fn new(user_id: String) -> Self {
        Self {
            user_id,
            email: None,
            roles: Vec::new(),
            permissions: Vec::new(),
            metadata: serde_json::Value::Null,
        }
    }

    /// Set email
    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    /// Add a role
    pub fn with_role(mut self, role: String) -> Self {
        self.roles.push(role);
        self
    }

    /// Add roles
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Add a permission
    pub fn with_permission(mut self, permission: String) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Add permissions
    pub fn with_permissions(mut self, permissions: Vec<String>) -> Self {
        self.permissions = permissions;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl AuthUser for UserContext {
    fn user_id(&self) -> String {
        self.user_id.clone()
    }

    fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_context() {
        let user = UserContext::new("user123".to_string())
            .with_email("user@example.com".to_string())
            .with_roles(vec!["admin".to_string(), "user".to_string()])
            .with_permissions(vec!["read".to_string(), "write".to_string()]);

        assert_eq!(user.user_id(), "user123");
        assert!(user.has_role("admin"));
        assert!(user.has_role("user"));
        assert!(!user.has_role("guest"));
        assert!(user.has_any_role(&["admin", "guest"]));
        assert!(!user.has_all_roles(&["admin", "guest"]));
        assert!(user.has_permission("read"));
        assert!(user.has_permission("write"));
        assert!(!user.has_permission("delete"));
    }
}
