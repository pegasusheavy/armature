//! Security middleware for Armature - inspired by Helmet for Express.js
//!
//! This module provides a comprehensive set of security headers and protections
//! to help secure your Armature applications against common web vulnerabilities.
//!
//! # Example
//!
//! ```
//! use armature_security::SecurityMiddleware;
//! use armature_security::hsts::HstsConfig;
//! use armature_security::frame_guard::FrameGuard;
//!
//! // Use default security settings (recommended)
//! let security = SecurityMiddleware::default();
//!
//! // Or customize as needed
//! let security = SecurityMiddleware::new()
//!     .with_hsts(HstsConfig::new(31536000))
//!     .with_frame_guard(FrameGuard::Deny)
//!     .hide_powered_by(true);
//! ```

pub mod content_security_policy;
pub mod dns_prefetch_control;
pub mod expect_ct;
pub mod frame_guard;
pub mod hsts;
pub mod powered_by;
pub mod referrer_policy;
pub mod xss_filter;
pub mod content_type_options;
pub mod download_options;
pub mod permitted_cross_domain_policies;

use armature_core::HttpResponse;
use std::collections::HashMap;

/// Main security middleware that combines all security features
#[derive(Debug, Clone)]
pub struct SecurityMiddleware {
    /// Content Security Policy configuration
    pub csp: Option<content_security_policy::CspConfig>,
    
    /// DNS Prefetch Control
    pub dns_prefetch_control: dns_prefetch_control::DnsPrefetchControl,
    
    /// Expect-CT configuration
    pub expect_ct: Option<expect_ct::ExpectCtConfig>,
    
    /// Frame Guard (X-Frame-Options)
    pub frame_guard: frame_guard::FrameGuard,
    
    /// HSTS (Strict-Transport-Security)
    pub hsts: Option<hsts::HstsConfig>,
    
    /// Hide X-Powered-By header
    pub hide_powered_by: bool,
    
    /// Referrer Policy
    pub referrer_policy: referrer_policy::ReferrerPolicy,
    
    /// X-XSS-Protection
    pub xss_filter: xss_filter::XssFilter,
    
    /// X-Content-Type-Options
    pub content_type_options: content_type_options::ContentTypeOptions,
    
    /// X-Download-Options
    pub download_options: download_options::DownloadOptions,
    
    /// X-Permitted-Cross-Domain-Policies
    pub permitted_cross_domain_policies: permitted_cross_domain_policies::PermittedCrossDomainPolicies,
}

impl SecurityMiddleware {
    /// Create a new security middleware with no protections enabled
    pub fn new() -> Self {
        Self {
            csp: None,
            dns_prefetch_control: dns_prefetch_control::DnsPrefetchControl::Off,
            expect_ct: None,
            frame_guard: frame_guard::FrameGuard::Deny,
            hsts: None,
            hide_powered_by: false,
            referrer_policy: referrer_policy::ReferrerPolicy::NoReferrer,
            xss_filter: xss_filter::XssFilter::Enabled,
            content_type_options: content_type_options::ContentTypeOptions::NoSniff,
            download_options: download_options::DownloadOptions::NoOpen,
            permitted_cross_domain_policies: permitted_cross_domain_policies::PermittedCrossDomainPolicies::None,
        }
    }
    
    /// Enable Content Security Policy
    pub fn with_csp(mut self, config: content_security_policy::CspConfig) -> Self {
        self.csp = Some(config);
        self
    }
    
    /// Enable DNS Prefetch Control
    pub fn with_dns_prefetch_control(mut self, control: dns_prefetch_control::DnsPrefetchControl) -> Self {
        self.dns_prefetch_control = control;
        self
    }
    
    /// Enable Expect-CT
    pub fn with_expect_ct(mut self, config: expect_ct::ExpectCtConfig) -> Self {
        self.expect_ct = Some(config);
        self
    }
    
    /// Set Frame Guard policy
    pub fn with_frame_guard(mut self, guard: frame_guard::FrameGuard) -> Self {
        self.frame_guard = guard;
        self
    }
    
    /// Enable HSTS
    pub fn with_hsts(mut self, config: hsts::HstsConfig) -> Self {
        self.hsts = Some(config);
        self
    }
    
    /// Hide X-Powered-By header
    pub fn hide_powered_by(mut self, hide: bool) -> Self {
        self.hide_powered_by = hide;
        self
    }
    
    /// Set Referrer Policy
    pub fn with_referrer_policy(mut self, policy: referrer_policy::ReferrerPolicy) -> Self {
        self.referrer_policy = policy;
        self
    }
    
    /// Set XSS Filter
    pub fn with_xss_filter(mut self, filter: xss_filter::XssFilter) -> Self {
        self.xss_filter = filter;
        self
    }
    
    /// Apply security headers to a response
    pub fn apply(&self, mut response: HttpResponse) -> HttpResponse {
        let mut headers = HashMap::new();
        
        // Content Security Policy
        if let Some(ref csp) = self.csp {
            headers.insert("Content-Security-Policy".to_string(), csp.to_header_value());
        }
        
        // DNS Prefetch Control
        headers.insert("X-DNS-Prefetch-Control".to_string(), self.dns_prefetch_control.to_header_value());
        
        // Expect-CT
        if let Some(ref expect_ct) = self.expect_ct {
            headers.insert("Expect-CT".to_string(), expect_ct.to_header_value());
        }
        
        // Frame Guard
        headers.insert("X-Frame-Options".to_string(), self.frame_guard.to_header_value());
        
        // HSTS
        if let Some(ref hsts) = self.hsts {
            headers.insert("Strict-Transport-Security".to_string(), hsts.to_header_value());
        }
        
        // Referrer Policy
        headers.insert("Referrer-Policy".to_string(), self.referrer_policy.to_header_value());
        
        // XSS Filter
        headers.insert("X-XSS-Protection".to_string(), self.xss_filter.to_header_value());
        
        // Content Type Options
        headers.insert("X-Content-Type-Options".to_string(), self.content_type_options.to_header_value());
        
        // Download Options
        headers.insert("X-Download-Options".to_string(), self.download_options.to_header_value());
        
        // Permitted Cross Domain Policies
        headers.insert("X-Permitted-Cross-Domain-Policies".to_string(), self.permitted_cross_domain_policies.to_header_value());
        
        // Apply all headers
        for (key, value) in headers {
            response.headers.insert(key, value);
        }
        
        // Remove X-Powered-By if requested
        if self.hide_powered_by {
            response.headers.remove("X-Powered-By");
        }
        
        response
    }
    
    /// Convenience method to enable all common security features (recommended defaults)
    pub fn enable_all(max_age_seconds: u64) -> Self {
        Self {
            csp: Some(content_security_policy::CspConfig::default()),
            dns_prefetch_control: dns_prefetch_control::DnsPrefetchControl::Off,
            expect_ct: Some(expect_ct::ExpectCtConfig::new(max_age_seconds)),
            frame_guard: frame_guard::FrameGuard::Deny,
            hsts: Some(hsts::HstsConfig::new(max_age_seconds)),
            hide_powered_by: true,
            referrer_policy: referrer_policy::ReferrerPolicy::NoReferrer,
            xss_filter: xss_filter::XssFilter::Enabled,
            content_type_options: content_type_options::ContentTypeOptions::NoSniff,
            download_options: download_options::DownloadOptions::NoOpen,
            permitted_cross_domain_policies: permitted_cross_domain_policies::PermittedCrossDomainPolicies::None,
        }
    }
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::enable_all(31536000) // 1 year
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_middleware_new() {
        let middleware = SecurityMiddleware::new();
        assert!(middleware.csp.is_none());
        assert!(!middleware.hide_powered_by);
    }

    #[test]
    fn test_security_middleware_default() {
        let middleware = SecurityMiddleware::default();
        assert!(middleware.csp.is_some());
        assert!(middleware.hsts.is_some());
        assert!(middleware.hide_powered_by);
    }

    #[test]
    fn test_security_middleware_apply() {
        let middleware = SecurityMiddleware::default();
        let response = HttpResponse::ok();
        let secured = middleware.apply(response);

        assert!(secured.headers.contains_key("X-Frame-Options"));
        assert!(secured.headers.contains_key("X-Content-Type-Options"));
        assert!(secured.headers.contains_key("X-XSS-Protection"));
        assert!(secured.headers.contains_key("Strict-Transport-Security"));
        assert!(secured.headers.contains_key("Content-Security-Policy"));
    }

    #[test]
    fn test_hide_powered_by() {
        let middleware = SecurityMiddleware::new().hide_powered_by(true);
        let mut response = HttpResponse::ok();
        response.headers.insert("X-Powered-By".to_string(), "Armature".to_string());
        
        let secured = middleware.apply(response);
        assert!(!secured.headers.contains_key("X-Powered-By"));
    }

    #[test]
    fn test_custom_configuration() {
        let middleware = SecurityMiddleware::new()
            .with_frame_guard(frame_guard::FrameGuard::SameOrigin)
            .with_referrer_policy(referrer_policy::ReferrerPolicy::StrictOriginWhenCrossOrigin)
            .hide_powered_by(true);

        let response = HttpResponse::ok();
        let secured = middleware.apply(response);

        assert_eq!(secured.headers.get("X-Frame-Options"), Some(&"SAMEORIGIN".to_string()));
        assert_eq!(secured.headers.get("Referrer-Policy"), Some(&"strict-origin-when-cross-origin".to_string()));
    }
}

