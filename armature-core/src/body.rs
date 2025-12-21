//! Zero-Copy HTTP Body Handling
//!
//! This module provides efficient body types that use `bytes::Bytes` internally
//! for zero-copy operations. By avoiding `Vec<u8>` conversions, request and
//! response bodies can be passed through without copying.
//!
//! ## Performance
//!
//! - **Request body**: Hyper's body is collected to `Bytes` once, no further copies
//! - **Response body**: `Bytes` is passed directly to Hyper, no conversion needed
//! - **Cloning**: `Bytes::clone()` is O(1) - just increments reference count
//!
//! ## Usage
//!
//! ```rust,ignore
//! use armature_core::body::{RequestBody, ResponseBody};
//!
//! // Request body - zero-copy from Hyper
//! let body = RequestBody::from_hyper(hyper_body).await?;
//! let json: MyType = body.json()?;
//!
//! // Response body - zero-copy to Hyper
//! let response = ResponseBody::from_json(&data)?;
//! let hyper_body = response.into_hyper(); // No copy!
//! ```

use bytes::Bytes;
use http_body_util::Full;
use serde::{de::DeserializeOwned, Serialize};
use std::ops::Deref;

// Re-export Bytes for convenience
pub use bytes;

/// A request body backed by `Bytes` for zero-copy handling.
///
/// This wraps `bytes::Bytes` to provide efficient body handling
/// without copying data from Hyper's incoming body.
///
/// # Example
///
/// ```rust,ignore
/// // From Hyper body (zero-copy after initial collect)
/// let body = RequestBody::from_bytes(hyper_bytes);
///
/// // Access as slice (zero-copy)
/// let slice: &[u8] = body.as_ref();
///
/// // Parse as JSON (zero-copy read)
/// let data: MyType = body.json()?;
/// ```
#[derive(Clone, Default)]
pub struct RequestBody {
    inner: Bytes,
}

impl RequestBody {
    /// Create an empty request body.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }

    /// Create from `Bytes` (zero-copy).
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }

    /// Create from a byte slice (copies data).
    ///
    /// Use `from_bytes` when possible to avoid copying.
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }

    /// Create from a `Vec<u8>` (zero-copy conversion).
    #[inline]
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }

    /// Create from a static byte array (zero-copy).
    #[inline]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(bytes),
        }
    }

    /// Get the body length.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the body is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the underlying `Bytes`.
    #[inline]
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Convert to `Bytes` (zero-copy).
    #[inline]
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// Convert to `Vec<u8>` (may copy if shared).
    ///
    /// If this is the only reference to the data, this is O(1).
    /// If the data is shared, this will copy.
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.inner.to_vec()
    }

    /// Parse body as JSON.
    ///
    /// Uses SIMD-accelerated parsing when the `simd-json` feature is enabled.
    #[inline]
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, crate::Error> {
        crate::json::from_slice(&self.inner).map_err(|e| crate::Error::Deserialization(e.to_string()))
    }

    /// Parse body as URL-encoded form data.
    #[inline]
    pub fn form<T: DeserializeOwned>(&self) -> Result<T, crate::Error> {
        crate::form::parse_form(&self.inner)
    }

    /// Get body as UTF-8 string (zero-copy if valid UTF-8).
    #[inline]
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.inner)
    }

    /// Get body as UTF-8 string, replacing invalid sequences.
    #[inline]
    pub fn to_string_lossy(&self) -> std::borrow::Cow<'_, str> {
        String::from_utf8_lossy(&self.inner)
    }
}

impl Deref for RequestBody {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for RequestBody {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Bytes> for RequestBody {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Vec<u8>> for RequestBody {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<&'static [u8]> for RequestBody {
    #[inline]
    fn from(slice: &'static [u8]) -> Self {
        Self::from_static(slice)
    }
}

impl From<String> for RequestBody {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_vec(s.into_bytes())
    }
}

impl From<&'static str> for RequestBody {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::from_static(s.as_bytes())
    }
}

impl std::fmt::Debug for RequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestBody")
            .field("len", &self.inner.len())
            .finish()
    }
}

// ============================================================================
// Response Body
// ============================================================================

/// A response body backed by `Bytes` for zero-copy handling.
///
/// This wraps `bytes::Bytes` to provide efficient body handling
/// that can be passed directly to Hyper without copying.
///
/// # Example
///
/// ```rust,ignore
/// // Create from JSON (serializes once)
/// let body = ResponseBody::from_json(&data)?;
///
/// // Convert to Hyper body (zero-copy)
/// let hyper_body: Full<Bytes> = body.into_hyper();
/// ```
#[derive(Clone, Default)]
pub struct ResponseBody {
    inner: Bytes,
}

impl ResponseBody {
    /// Create an empty response body.
    #[inline]
    pub const fn empty() -> Self {
        Self {
            inner: Bytes::new(),
        }
    }

    /// Create from `Bytes` (zero-copy).
    #[inline]
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self { inner: bytes }
    }

    /// Create from a byte slice (copies data).
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: Bytes::copy_from_slice(slice),
        }
    }

    /// Create from a `Vec<u8>` (zero-copy conversion).
    #[inline]
    pub fn from_vec(vec: Vec<u8>) -> Self {
        Self {
            inner: Bytes::from(vec),
        }
    }

    /// Create from a static byte array (zero-copy).
    #[inline]
    pub fn from_static(bytes: &'static [u8]) -> Self {
        Self {
            inner: Bytes::from_static(bytes),
        }
    }

    /// Create from JSON serialization.
    ///
    /// Uses SIMD-accelerated serialization when the `simd-json` feature is enabled.
    #[inline]
    pub fn from_json<T: Serialize>(value: &T) -> Result<Self, crate::Error> {
        let vec = crate::json::to_vec(value).map_err(|e| crate::Error::Serialization(e.to_string()))?;
        Ok(Self::from_vec(vec))
    }

    /// Create from JSON with pre-allocated capacity.
    ///
    /// Use this when you have a reasonable estimate of the output size.
    #[inline]
    pub fn from_json_with_capacity<T: Serialize>(value: &T, capacity: usize) -> Result<Self, crate::Error> {
        let vec = crate::json::to_vec_with_capacity(value, capacity)
            .map_err(|e| crate::Error::Serialization(e.to_string()))?;
        Ok(Self::from_vec(vec))
    }

    /// Get the body length.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the body is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the underlying `Bytes`.
    #[inline]
    pub fn as_bytes(&self) -> &Bytes {
        &self.inner
    }

    /// Convert to `Bytes` (zero-copy).
    #[inline]
    pub fn into_bytes(self) -> Bytes {
        self.inner
    }

    /// Convert to Hyper's body type (zero-copy).
    ///
    /// This is the key optimization - no copying needed!
    #[inline]
    pub fn into_hyper(self) -> Full<Bytes> {
        Full::new(self.inner)
    }

    /// Convert to `Vec<u8>` (may copy if shared).
    #[inline]
    pub fn to_vec(&self) -> Vec<u8> {
        self.inner.to_vec()
    }
}

impl Deref for ResponseBody {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AsRef<[u8]> for ResponseBody {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        &self.inner
    }
}

impl From<Bytes> for ResponseBody {
    #[inline]
    fn from(bytes: Bytes) -> Self {
        Self::from_bytes(bytes)
    }
}

impl From<Vec<u8>> for ResponseBody {
    #[inline]
    fn from(vec: Vec<u8>) -> Self {
        Self::from_vec(vec)
    }
}

impl From<&'static [u8]> for ResponseBody {
    #[inline]
    fn from(slice: &'static [u8]) -> Self {
        Self::from_static(slice)
    }
}

impl From<String> for ResponseBody {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_vec(s.into_bytes())
    }
}

impl From<&'static str> for ResponseBody {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::from_static(s.as_bytes())
    }
}

impl std::fmt::Debug for ResponseBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResponseBody")
            .field("len", &self.inner.len())
            .finish()
    }
}

// ============================================================================
// Conversion to/from legacy types
// ============================================================================

impl From<RequestBody> for Vec<u8> {
    #[inline]
    fn from(body: RequestBody) -> Self {
        body.to_vec()
    }
}

impl From<ResponseBody> for Vec<u8> {
    #[inline]
    fn from(body: ResponseBody) -> Self {
        body.to_vec()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_body_from_bytes() {
        let bytes = Bytes::from_static(b"hello world");
        let body = RequestBody::from_bytes(bytes);
        assert_eq!(body.len(), 11);
        assert_eq!(&*body, b"hello world");
    }

    #[test]
    fn test_request_body_from_vec() {
        let vec = vec![1, 2, 3, 4, 5];
        let body = RequestBody::from_vec(vec);
        assert_eq!(body.len(), 5);
        assert_eq!(&*body, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_request_body_json() {
        let json = br#"{"name":"John","age":30}"#;
        let body = RequestBody::from_slice(json);

        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Person {
            name: String,
            age: u32,
        }

        let person: Person = body.json().unwrap();
        assert_eq!(person.name, "John");
        assert_eq!(person.age, 30);
    }

    #[test]
    fn test_request_body_clone_is_cheap() {
        let body = RequestBody::from_static(b"large data here that would be expensive to copy");
        let _clone1 = body.clone(); // Should be O(1) - just ref count increment
        let _clone2 = body.clone();
        assert_eq!(body.len(), 47);
    }

    #[test]
    fn test_response_body_from_json() {
        #[derive(serde::Serialize)]
        struct Response {
            status: &'static str,
            code: u32,
        }

        let data = Response {
            status: "ok",
            code: 200,
        };

        let body = ResponseBody::from_json(&data).unwrap();
        assert!(!body.is_empty());
        assert!(String::from_utf8_lossy(&body).contains("ok"));
    }

    #[test]
    fn test_response_body_into_hyper() {
        let body = ResponseBody::from_static(b"response content");
        let hyper_body = body.into_hyper();
        // Full<Bytes> is the type Hyper expects - zero copy!
        let _ = hyper_body;
    }

    #[test]
    fn test_response_body_from_string() {
        let body = ResponseBody::from("hello world".to_string());
        assert_eq!(&*body, b"hello world");
    }

    #[test]
    fn test_request_body_as_str() {
        let body = RequestBody::from_static(b"valid utf-8");
        assert_eq!(body.as_str().unwrap(), "valid utf-8");

        let invalid = RequestBody::from_slice(&[0xff, 0xfe]);
        assert!(invalid.as_str().is_err());
    }

    #[test]
    fn test_empty_bodies() {
        let req = RequestBody::empty();
        assert!(req.is_empty());
        assert_eq!(req.len(), 0);

        let resp = ResponseBody::empty();
        assert!(resp.is_empty());
        assert_eq!(resp.len(), 0);
    }
}

