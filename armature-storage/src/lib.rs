//! File storage and upload handling for Armature
//!
//! This module provides:
//! - Multipart file upload handling
//! - File validation (type, size, extension)
//! - Multiple storage backends (Local, S3, GCS, Azure Blob)
//! - Dependency injection for cloud providers
//! - Production-ready file management
//!
//! # Features
//!
//! - **Multipart Upload** - Handle file uploads from forms
//! - **Validation** - Type, size, extension validation
//! - **Local Storage** - Filesystem storage
//! - **S3 Storage** - AWS S3 (with DI)
//! - **GCS Storage** - Google Cloud Storage (with DI)
//! - **Azure Blob** - Azure Blob Storage (with DI)
//!
//! # Quick Start
//!
//! ```no_run
//! use armature_storage::*;
//!
//! # async fn example() -> Result<(), StorageError> {
//! // Create local storage
//! let storage = LocalStorage::new("./uploads");
//!
//! // Upload a file
//! let metadata = storage.put("user-avatar.jpg", b"file data").await?;
//! println!("Uploaded: {}", metadata.key);
//! # Ok(())
//! # }
//! ```

pub mod storage;
pub mod validation;
pub mod multipart;
pub mod local;

#[cfg(feature = "s3")]
pub mod s3;

#[cfg(feature = "gcs")]
pub mod gcs;

#[cfg(feature = "azure")]
pub mod azure;

pub use storage::*;
pub use validation::*;
pub use multipart::*;
pub use local::*;

#[cfg(feature = "s3")]
pub use s3::*;

#[cfg(feature = "gcs")]
pub use gcs::*;

#[cfg(feature = "azure")]
pub use azure::*;
