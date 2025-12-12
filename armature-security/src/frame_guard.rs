//! Frame Guard (X-Frame-Options)
//!
//! Mitigates clickjacking attacks.

/// Frame Guard options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameGuard {
    /// Deny all framing
    Deny,
    /// Allow framing from same origin
    SameOrigin,
    /// Allow framing from specific origin
    AllowFrom(String),
}

impl FrameGuard {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::Deny => "DENY".to_string(),
            Self::SameOrigin => "SAMEORIGIN".to_string(),
            Self::AllowFrom(origin) => format!("ALLOW-FROM {}", origin),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_guard() {
        assert_eq!(FrameGuard::Deny.to_header_value(), "DENY");
        assert_eq!(FrameGuard::SameOrigin.to_header_value(), "SAMEORIGIN");
        assert_eq!(
            FrameGuard::AllowFrom("https://example.com".to_string()).to_header_value(),
            "ALLOW-FROM https://example.com"
        );
    }
}
