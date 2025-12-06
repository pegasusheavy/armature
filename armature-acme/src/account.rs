/// ACME account management
use serde::{Deserialize, Serialize};

/// ACME account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Account status
    pub status: AccountStatus,

    /// Contact information (email addresses)
    pub contact: Vec<String>,

    /// Account URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orders: Option<String>,

    /// Terms of service agreement
    #[serde(rename = "termsOfServiceAgreed", skip_serializing_if = "Option::is_none")]
    pub terms_of_service_agreed: Option<bool>,

    /// External account binding
    #[serde(rename = "externalAccountBinding", skip_serializing_if = "Option::is_none")]
    pub external_account_binding: Option<serde_json::Value>,
}

/// Account status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum AccountStatus {
    /// Account is valid
    Valid,
    /// Account is deactivated
    Deactivated,
    /// Account is revoked
    Revoked,
}

/// Account creation request
#[derive(Debug, Clone, Serialize)]
pub struct AccountCreate {
    /// Contact information
    pub contact: Vec<String>,

    /// Terms of service agreement
    #[serde(rename = "termsOfServiceAgreed")]
    pub terms_of_service_agreed: bool,

    /// External account binding (for providers that require it)
    #[serde(rename = "externalAccountBinding", skip_serializing_if = "Option::is_none")]
    pub external_account_binding: Option<serde_json::Value>,
}

impl AccountCreate {
    /// Create a new account creation request
    pub fn new(contact: Vec<String>, terms_of_service_agreed: bool) -> Self {
        Self {
            contact,
            terms_of_service_agreed,
            external_account_binding: None,
        }
    }

    /// Add external account binding
    pub fn with_eab(mut self, eab: serde_json::Value) -> Self {
        self.external_account_binding = Some(eab);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_create() {
        let account_create = AccountCreate::new(
            vec!["mailto:admin@example.com".to_string()],
            true,
        );

        assert_eq!(account_create.contact.len(), 1);
        assert!(account_create.terms_of_service_agreed);
        assert!(account_create.external_account_binding.is_none());
    }

    #[test]
    fn test_account_status_serialization() {
        let status = AccountStatus::Valid;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"valid\"");
    }
}

