//! Referrer Policy
//!
//! Controls how much referrer information is included with requests.

/// Referrer Policy options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferrerPolicy {
    /// No referrer information
    NoReferrer,
    /// No referrer when downgrading from HTTPS to HTTP
    NoReferrerWhenDowngrade,
    /// Only send origin
    Origin,
    /// Only send origin for cross-origin requests
    OriginWhenCrossOrigin,
    /// Only same origin
    SameOrigin,
    /// Only send origin when protocol security level stays the same
    StrictOrigin,
    /// Send full URL for same-origin, origin only for cross-origin if protocol security stays same
    StrictOriginWhenCrossOrigin,
    /// Always send full URL
    UnsafeUrl,
}

impl ReferrerPolicy {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::NoReferrer => "no-referrer",
            Self::NoReferrerWhenDowngrade => "no-referrer-when-downgrade",
            Self::Origin => "origin",
            Self::OriginWhenCrossOrigin => "origin-when-cross-origin",
            Self::SameOrigin => "same-origin",
            Self::StrictOrigin => "strict-origin",
            Self::StrictOriginWhenCrossOrigin => "strict-origin-when-cross-origin",
            Self::UnsafeUrl => "unsafe-url",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_referrer_policy() {
        assert_eq!(ReferrerPolicy::NoReferrer.to_header_value(), "no-referrer");
        assert_eq!(
            ReferrerPolicy::StrictOriginWhenCrossOrigin.to_header_value(),
            "strict-origin-when-cross-origin"
        );
        assert_eq!(ReferrerPolicy::SameOrigin.to_header_value(), "same-origin");
    }
}
