//! X-Download-Options
//!
//! Prevents Internet Explorer from executing downloads in the site's context.

/// Download Options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadOptions {
    /// Prevent opening downloads in context
    NoOpen,
}

impl DownloadOptions {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::NoOpen => "noopen".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_options() {
        assert_eq!(DownloadOptions::NoOpen.to_header_value(), "noopen");
    }
}
