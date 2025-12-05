//! X-Permitted-Cross-Domain-Policies
//!
//! Controls cross-domain policies for Adobe products.

/// Permitted Cross Domain Policies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermittedCrossDomainPolicies {
    /// No cross-domain policies allowed
    None,
    /// Only master policy allowed
    MasterOnly,
    /// Only policies from this domain
    ByContentType,
    /// All cross-domain policies allowed
    All,
}

impl PermittedCrossDomainPolicies {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::None => "none",
            Self::MasterOnly => "master-only",
            Self::ByContentType => "by-content-type",
            Self::All => "all",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permitted_cross_domain_policies() {
        assert_eq!(PermittedCrossDomainPolicies::None.to_header_value(), "none");
        assert_eq!(PermittedCrossDomainPolicies::MasterOnly.to_header_value(), "master-only");
        assert_eq!(PermittedCrossDomainPolicies::All.to_header_value(), "all");
    }
}

