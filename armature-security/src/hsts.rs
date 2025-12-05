//! HTTP Strict Transport Security (HSTS)
//!
//! Forces browsers to use HTTPS.

/// HSTS configuration
#[derive(Debug, Clone)]
pub struct HstsConfig {
    /// Max age in seconds
    pub max_age: u64,
    
    /// Include subdomains
    pub include_subdomains: bool,
    
    /// Preload (submit to browser preload list)
    pub preload: bool,
}

impl HstsConfig {
    /// Create a new HSTS configuration
    pub fn new(max_age: u64) -> Self {
        Self {
            max_age,
            include_subdomains: true,
            preload: false,
        }
    }
    
    /// Include subdomains
    pub fn include_subdomains(mut self, include: bool) -> Self {
        self.include_subdomains = include;
        self
    }
    
    /// Enable preloading
    pub fn preload(mut self, preload: bool) -> Self {
        self.preload = preload;
        self
    }
    
    /// Convert to header value
    pub fn to_header_value(&self) -> String {
        let mut parts = vec![format!("max-age={}", self.max_age)];
        
        if self.include_subdomains {
            parts.push("includeSubDomains".to_string());
        }
        
        if self.preload {
            parts.push("preload".to_string());
        }
        
        parts.join("; ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsts_basic() {
        let config = HstsConfig::new(31536000);
        assert_eq!(config.to_header_value(), "max-age=31536000; includeSubDomains");
    }

    #[test]
    fn test_hsts_no_subdomains() {
        let config = HstsConfig::new(31536000).include_subdomains(false);
        assert_eq!(config.to_header_value(), "max-age=31536000");
    }

    #[test]
    fn test_hsts_preload() {
        let config = HstsConfig::new(31536000).preload(true);
        assert_eq!(config.to_header_value(), "max-age=31536000; includeSubDomains; preload");
    }
}

