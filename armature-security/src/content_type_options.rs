//! X-Content-Type-Options
//!
//! Prevents browsers from MIME-sniffing away from declared content type.

/// Content Type Options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentTypeOptions {
    /// Prevent MIME sniffing
    NoSniff,
}

impl ContentTypeOptions {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::NoSniff => "nosniff".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_options() {
        assert_eq!(ContentTypeOptions::NoSniff.to_header_value(), "nosniff");
    }
}

