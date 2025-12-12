//! DNS Prefetch Control
//!
//! Controls browser DNS prefetching to improve privacy.

/// DNS Prefetch Control options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsPrefetchControl {
    /// Disable DNS prefetching
    Off,
    /// Enable DNS prefetching
    On,
}

impl DnsPrefetchControl {
    pub fn to_header_value(&self) -> String {
        match self {
            Self::Off => "off".to_string(),
            Self::On => "on".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_prefetch_control() {
        assert_eq!(DnsPrefetchControl::Off.to_header_value(), "off");
        assert_eq!(DnsPrefetchControl::On.to_header_value(), "on");
    }
}
