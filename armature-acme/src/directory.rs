/// ACME directory and metadata
use serde::{Deserialize, Serialize};

/// ACME directory structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directory {
    /// URL for creating new accounts
    #[serde(rename = "newAccount")]
    pub new_account: String,

    /// URL for creating new orders
    #[serde(rename = "newOrder")]
    pub new_order: String,

    /// URL for creating new nonces
    #[serde(rename = "newNonce")]
    pub new_nonce: String,

    /// URL for revoking certificates
    #[serde(rename = "revokeCert")]
    pub revoke_cert: String,

    /// Optional: URL for key change
    #[serde(rename = "keyChange", skip_serializing_if = "Option::is_none")]
    pub key_change: Option<String>,

    /// Optional: Directory metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<DirectoryMeta>,
}

/// ACME directory metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryMeta {
    /// Terms of service URL
    #[serde(rename = "termsOfService", skip_serializing_if = "Option::is_none")]
    pub terms_of_service: Option<String>,

    /// Website URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub website: Option<String>,

    /// CAA identities
    #[serde(rename = "caaIdentities", skip_serializing_if = "Option::is_none")]
    pub caa_identities: Option<Vec<String>>,

    /// Whether external account binding is required
    #[serde(
        rename = "externalAccountRequired",
        skip_serializing_if = "Option::is_none"
    )]
    pub external_account_required: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directory_deserialization() {
        let json = r#"{
            "newAccount": "https://example.com/acme/new-account",
            "newOrder": "https://example.com/acme/new-order",
            "newNonce": "https://example.com/acme/new-nonce",
            "revokeCert": "https://example.com/acme/revoke-cert"
        }"#;

        let directory: Directory = serde_json::from_str(json).unwrap();
        assert_eq!(
            directory.new_account,
            "https://example.com/acme/new-account"
        );
        assert_eq!(directory.new_order, "https://example.com/acme/new-order");
    }

    #[test]
    fn test_directory_with_meta() {
        let json = r#"{
            "newAccount": "https://example.com/acme/new-account",
            "newOrder": "https://example.com/acme/new-order",
            "newNonce": "https://example.com/acme/new-nonce",
            "revokeCert": "https://example.com/acme/revoke-cert",
            "meta": {
                "termsOfService": "https://example.com/tos",
                "externalAccountRequired": true
            }
        }"#;

        let directory: Directory = serde_json::from_str(json).unwrap();
        assert!(directory.meta.is_some());
        let meta = directory.meta.unwrap();
        assert_eq!(meta.terms_of_service, Some("https://example.com/tos".to_string()));
        assert_eq!(meta.external_account_required, Some(true));
    }
}

