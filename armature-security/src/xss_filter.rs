//! X-XSS-Protection
//!
//! Enables the Cross-site scripting (XSS) filter built into browsers.

/// XSS Filter options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XssFilter {
    /// Disable XSS filtering
    Disabled,
    /// Enable XSS filtering
    Enabled,
    /// Enable XSS filtering and block page if attack detected
    EnabledBlock,
}

impl XssFilter {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::Disabled => "0",
            Self::Enabled => "1",
            Self::EnabledBlock => "1; mode=block",
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xss_filter() {
        assert_eq!(XssFilter::Disabled.to_header_value(), "0");
        assert_eq!(XssFilter::Enabled.to_header_value(), "1");
        assert_eq!(XssFilter::EnabledBlock.to_header_value(), "1; mode=block");
    }
}
