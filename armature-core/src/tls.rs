//! TLS/HTTPS support for Armature
//!
//! This module provides TLS certificate management and HTTPS server configuration.

use crate::Error;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use rustls_pemfile::{certs, private_key};
use std::{fs::File, io::BufReader, path::Path, sync::Arc};

/// TLS configuration for HTTPS server
#[derive(Clone, Debug)]
pub struct TlsConfig {
    /// Server configuration
    pub server_config: Arc<ServerConfig>,
}

impl TlsConfig {
    /// Create a new TLS configuration from certificate and key files
    ///
    /// # Arguments
    ///
    /// * `cert_path` - Path to the certificate file (PEM format)
    /// * `key_path` - Path to the private key file (PEM format)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use armature_core::tls::TlsConfig;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tls_config = TlsConfig::from_pem_files("cert.pem", "key.pem")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_pem_files(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> Result<Self, Error> {
        let certs = load_certs(cert_path.as_ref())?;
        let key = load_private_key(key_path.as_ref())?;

        Self::from_pem_parts(certs, key)
    }

    /// Create a new TLS configuration from certificate and key bytes
    ///
    /// # Arguments
    ///
    /// * `cert_pem` - Certificate in PEM format
    /// * `key_pem` - Private key in PEM format
    pub fn from_pem_bytes(cert_pem: &[u8], key_pem: &[u8]) -> Result<Self, Error> {
        let certs = parse_certs(cert_pem)?;
        let key = parse_private_key(key_pem)?;

        Self::from_pem_parts(certs, key)
    }

    /// Create TLS configuration from certificate chain and private key
    fn from_pem_parts(
        certs: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<Self, Error> {
        let mut config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| Error::Internal(format!("Failed to create TLS config: {}", e)))?;

        // Enable HTTP/2 and HTTP/1.1
        config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(Self {
            server_config: Arc::new(config),
        })
    }

    /// Create a self-signed certificate for development/testing
    ///
    /// **WARNING**: This should NEVER be used in production!
    ///
    /// # Example
    ///
    /// ```
    /// use armature_core::tls::TlsConfig;
    ///
    /// # #[cfg(feature = "self-signed-certs")]
    /// # fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let tls_config = TlsConfig::self_signed(&["localhost", "127.0.0.1"])?;
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "self-signed-certs")]
    pub fn self_signed(domains: &[&str]) -> Result<Self, Error> {
        use rcgen::{CertificateParams, KeyPair};

        let mut params =
            CertificateParams::new(domains.iter().map(|s| s.to_string()).collect::<Vec<_>>());
        params.distinguished_name = rcgen::DistinguishedName::new();

        let key_pair = KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256)
            .map_err(|e| Error::Internal(format!("Failed to generate key pair: {}", e)))?;

        let cert = rcgen::Certificate::from_params(params)
            .map_err(|e| Error::Internal(format!("Failed to create certificate: {}", e)))?;

        let cert_pem = cert
            .serialize_pem()
            .map_err(|e| Error::Internal(format!("Failed to serialize certificate: {}", e)))?;

        let key_pem = key_pair.serialize_pem();

        Self::from_pem_bytes(cert_pem.as_bytes(), key_pem.as_bytes())
    }
}

/// Load certificates from a PEM file
fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>, Error> {
    let file = File::open(path)
        .map_err(|e| Error::Internal(format!("Failed to open certificate file: {}", e)))?;

    let mut reader = BufReader::new(file);

    certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Internal(format!("Failed to parse certificates: {}", e)))
}

/// Load private key from a PEM file
fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>, Error> {
    let file =
        File::open(path).map_err(|e| Error::Internal(format!("Failed to open key file: {}", e)))?;

    let mut reader = BufReader::new(file);

    private_key(&mut reader)
        .map_err(|e| Error::Internal(format!("Failed to read private key: {}", e)))?
        .ok_or_else(|| Error::Internal("No private key found in file".to_string()))
}

/// Parse certificates from PEM bytes
fn parse_certs(pem: &[u8]) -> Result<Vec<CertificateDer<'static>>, Error> {
    let mut reader = BufReader::new(pem);

    certs(&mut reader)
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| Error::Internal(format!("Failed to parse certificates: {}", e)))
}

/// Parse private key from PEM bytes
fn parse_private_key(pem: &[u8]) -> Result<PrivateKeyDer<'static>, Error> {
    let mut reader = BufReader::new(pem);

    private_key(&mut reader)
        .map_err(|e| Error::Internal(format!("Failed to read private key: {}", e)))?
        .ok_or_else(|| Error::Internal("No private key found".to_string()))
}

/// HTTPS server bind address configuration
#[derive(Debug, Clone)]
pub struct HttpsConfig {
    /// HTTPS bind address (e.g., "0.0.0.0:443")
    pub https_addr: String,

    /// Optional HTTP bind address for redirect (e.g., "0.0.0.0:80")
    pub http_redirect_addr: Option<String>,

    /// TLS configuration
    pub tls: TlsConfig,
}

impl HttpsConfig {
    /// Create a new HTTPS configuration
    pub fn new(https_addr: impl Into<String>, tls: TlsConfig) -> Self {
        Self {
            https_addr: https_addr.into(),
            http_redirect_addr: None,
            tls,
        }
    }

    /// Enable HTTP to HTTPS redirect
    pub fn with_http_redirect(mut self, http_addr: impl Into<String>) -> Self {
        self.http_redirect_addr = Some(http_addr.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "self-signed-certs")]
    fn test_self_signed_cert() {
        let config = TlsConfig::self_signed(&["localhost", "127.0.0.1"]).unwrap();
        assert!(
            config
                .server_config
                .alpn_protocols
                .contains(&b"h2".to_vec())
        );
        assert!(
            config
                .server_config
                .alpn_protocols
                .contains(&b"http/1.1".to_vec())
        );
    }

    #[test]
    fn test_https_config() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let https_config =
                HttpsConfig::new("0.0.0.0:443", tls).with_http_redirect("0.0.0.0:80");

            assert_eq!(https_config.https_addr, "0.0.0.0:443");
            assert_eq!(
                https_config.http_redirect_addr,
                Some("0.0.0.0:80".to_string())
            );
        }
    }

    #[test]
    fn test_https_config_without_redirect() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let https_config = HttpsConfig::new("0.0.0.0:8443", tls);

            assert_eq!(https_config.https_addr, "0.0.0.0:8443");
            assert!(https_config.http_redirect_addr.is_none());
        }
    }

    #[test]
    #[cfg(feature = "self-signed-certs")]
    fn test_self_signed_single_domain() {
        let tls = TlsConfig::self_signed(&["example.com"]);
        assert!(tls.is_ok());
    }

    #[test]
    #[cfg(feature = "self-signed-certs")]
    fn test_self_signed_multiple_domains() {
        let tls = TlsConfig::self_signed(&["example.com", "www.example.com", "api.example.com"]);
        assert!(tls.is_ok());
    }

    #[test]
    #[cfg(feature = "self-signed-certs")]
    fn test_self_signed_with_localhost() {
        let tls = TlsConfig::self_signed(&["localhost", "127.0.0.1"]);
        assert!(tls.is_ok());
    }

    #[test]
    fn test_from_pem_bytes_invalid_cert() {
        let invalid_cert = b"invalid certificate data";
        let valid_key = b"-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----";

        let result = TlsConfig::from_pem_bytes(invalid_cert, valid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_pem_bytes_invalid_key() {
        let cert = b"-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----";
        let invalid_key = b"invalid key data";

        let result = TlsConfig::from_pem_bytes(cert, invalid_key);
        assert!(result.is_err());
    }

    #[test]
    fn test_https_config_clone() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let config1 = HttpsConfig::new("0.0.0.0:443", tls);
            let config2 = config1.clone();

            assert_eq!(config1.https_addr, config2.https_addr);
            assert_eq!(config1.http_redirect_addr, config2.http_redirect_addr);
        }
    }

    #[test]
    fn test_https_config_with_different_ports() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let config = HttpsConfig::new("0.0.0.0:8443", tls).with_http_redirect("0.0.0.0:8080");

            assert_eq!(config.https_addr, "0.0.0.0:8443");
            assert_eq!(config.http_redirect_addr, Some("0.0.0.0:8080".to_string()));
        }
    }

    #[test]
    fn test_https_config_with_ipv6() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let config = HttpsConfig::new("[::1]:443", tls);

            assert_eq!(config.https_addr, "[::1]:443");
        }
    }

    #[test]
    fn test_tls_config_debug() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let debug_str = format!("{:?}", tls);
            assert!(debug_str.contains("TlsConfig"));
        }
    }

    #[test]
    fn test_https_config_debug() {
        #[cfg(feature = "self-signed-certs")]
        {
            let tls = TlsConfig::self_signed(&["localhost"]).unwrap();
            let config = HttpsConfig::new("0.0.0.0:443", tls);
            let debug_str = format!("{:?}", config);
            assert!(debug_str.contains("HttpsConfig"));
        }
    }
}
