//! Uploaded file types.

use bytes::Bytes;
use mime::Mime;
use std::path::Path;

use crate::{Result, StorageError};

/// Information about an uploaded file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    /// Original file name.
    pub name: Option<String>,
    /// File extension.
    pub extension: Option<String>,
    /// MIME type.
    pub content_type: Option<Mime>,
    /// File size in bytes.
    pub size: u64,
}

impl FileInfo {
    /// Create new file info.
    pub fn new() -> Self {
        Self {
            name: None,
            extension: None,
            content_type: None,
            size: 0,
        }
    }

    /// Get the file extension (lowercase).
    pub fn extension_lowercase(&self) -> Option<String> {
        self.extension.as_ref().map(|e| e.to_lowercase())
    }

    /// Check if the file has an image MIME type.
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.type_() == mime::IMAGE)
            .unwrap_or(false)
    }

    /// Check if the file has a video MIME type.
    pub fn is_video(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.type_() == mime::VIDEO)
            .unwrap_or(false)
    }

    /// Check if the file has an audio MIME type.
    pub fn is_audio(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.type_() == mime::AUDIO)
            .unwrap_or(false)
    }

    /// Check if the file has a text MIME type.
    pub fn is_text(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.type_() == mime::TEXT)
            .unwrap_or(false)
    }
}

impl Default for FileInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// An uploaded file with its data.
#[derive(Debug, Clone)]
pub struct UploadedFile {
    /// File information.
    pub info: FileInfo,
    /// File data.
    pub data: Bytes,
}

impl UploadedFile {
    /// Create a new uploaded file.
    pub fn new(data: Bytes) -> Self {
        Self {
            info: FileInfo {
                size: data.len() as u64,
                ..Default::default()
            },
            data,
        }
    }

    /// Create from a multipart field.
    pub async fn from_field(field: crate::MultipartField<'_>) -> Result<Self> {
        let name = field.file_name().map(String::from);
        let content_type = field.content_type().cloned();

        let extension = name.as_ref().and_then(|n| {
            Path::new(n)
                .extension()
                .map(|e| e.to_string_lossy().to_string())
        });

        let data = field.bytes().await.map_err(StorageError::from)?;
        let size = data.len() as u64;

        Ok(Self {
            info: FileInfo {
                name,
                extension,
                content_type,
                size,
            },
            data,
        })
    }

    /// Create from raw bytes with a name.
    pub fn from_bytes(data: impl Into<Bytes>, name: impl Into<String>) -> Self {
        let data = data.into();
        let name = name.into();
        let extension = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_string());
        let content_type = mime_guess::from_path(&name).first();

        Self {
            info: FileInfo {
                name: Some(name),
                extension,
                content_type,
                size: data.len() as u64,
            },
            data,
        }
    }

    /// Get the file name.
    pub fn name(&self) -> Option<&str> {
        self.info.name.as_deref()
    }

    /// Get the file extension.
    pub fn extension(&self) -> Option<&str> {
        self.info.extension.as_deref()
    }

    /// Get the content type.
    pub fn content_type(&self) -> Option<&Mime> {
        self.info.content_type.as_ref()
    }

    /// Get the content type as a string.
    pub fn content_type_str(&self) -> Option<String> {
        self.info.content_type.as_ref().map(|ct| ct.to_string())
    }

    /// Get the file size.
    pub fn size(&self) -> u64 {
        self.info.size
    }

    /// Get the file data.
    pub fn data(&self) -> &Bytes {
        &self.data
    }

    /// Consume and return the file data.
    pub fn into_data(self) -> Bytes {
        self.data
    }

    /// Check if the file is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Set the file name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.info.extension = Path::new(&name)
            .extension()
            .map(|e| e.to_string_lossy().to_string());
        self.info.name = Some(name);
        self
    }

    /// Set the content type.
    pub fn with_content_type(mut self, content_type: Mime) -> Self {
        self.info.content_type = Some(content_type);
        self
    }
}
