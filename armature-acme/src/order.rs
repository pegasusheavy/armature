/// ACME order management
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// ACME order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order status
    pub status: OrderStatus,

    /// Expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires: Option<DateTime<Utc>>,

    /// List of identifier objects
    pub identifiers: Vec<Identifier>,

    /// Authorization URLs
    pub authorizations: Vec<String>,

    /// Finalize URL
    pub finalize: String,

    /// Certificate URL (available when status is valid)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate: Option<String>,
}

/// Order status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    /// Order is pending authorization
    Pending,
    /// Order is ready for finalization
    Ready,
    /// Order is processing
    Processing,
    /// Order is valid and certificate is available
    Valid,
    /// Order is invalid
    Invalid,
}

/// Domain identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    /// Identifier type (usually "dns")
    #[serde(rename = "type")]
    pub id_type: String,

    /// Identifier value (domain name)
    pub value: String,
}

impl Identifier {
    /// Create a DNS identifier
    pub fn dns(domain: impl Into<String>) -> Self {
        Self {
            id_type: "dns".to_string(),
            value: domain.into(),
        }
    }
}

/// Order creation request
#[derive(Debug, Clone, Serialize)]
pub struct OrderCreate {
    /// List of identifiers to order
    pub identifiers: Vec<Identifier>,

    /// Optional: Not before timestamp
    #[serde(rename = "notBefore", skip_serializing_if = "Option::is_none")]
    pub not_before: Option<DateTime<Utc>>,

    /// Optional: Not after timestamp
    #[serde(rename = "notAfter", skip_serializing_if = "Option::is_none")]
    pub not_after: Option<DateTime<Utc>>,
}

impl OrderCreate {
    /// Create a new order for domains
    pub fn new(domains: Vec<String>) -> Self {
        Self {
            identifiers: domains.into_iter().map(Identifier::dns).collect(),
            not_before: None,
            not_after: None,
        }
    }

    /// Set not before timestamp
    pub fn with_not_before(mut self, timestamp: DateTime<Utc>) -> Self {
        self.not_before = Some(timestamp);
        self
    }

    /// Set not after timestamp
    pub fn with_not_after(mut self, timestamp: DateTime<Utc>) -> Self {
        self.not_after = Some(timestamp);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_dns() {
        let identifier = Identifier::dns("example.com");
        assert_eq!(identifier.id_type, "dns");
        assert_eq!(identifier.value, "example.com");
    }

    #[test]
    fn test_order_create() {
        let order = OrderCreate::new(vec![
            "example.com".to_string(),
            "www.example.com".to_string(),
        ]);

        assert_eq!(order.identifiers.len(), 2);
        assert_eq!(order.identifiers[0].value, "example.com");
        assert_eq!(order.identifiers[1].value, "www.example.com");
    }

    #[test]
    fn test_order_status_serialization() {
        let status = OrderStatus::Valid;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"valid\"");
    }
}

