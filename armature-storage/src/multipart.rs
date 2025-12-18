//! Multipart form data parsing.

use bytes::Bytes;
use futures::Stream;

use crate::{Result, StorageError, UploadedFile};

/// Re-export multer's Field type.
pub type MultipartField<'a> = multer::Field<'a>;

/// Multipart form data parser.
///
/// ## Example
///
/// ```rust,ignore
/// use armature_storage::{Multipart, UploadedFile};
///
/// async fn handle_upload(multipart: Multipart) -> Result<Vec<UploadedFile>, Error> {
///     let mut files = Vec::new();
///     let mut stream = multipart.into_stream();
///
///     while let Some(field) = stream.next_field().await? {
///         if let Some(filename) = field.file_name() {
///             let file = UploadedFile::from_field(field).await?;
///             files.push(file);
///         }
///     }
///
///     Ok(files)
/// }
/// ```
pub struct Multipart {
    inner: multer::Multipart<'static>,
}

impl Multipart {
    /// Create a new multipart parser from a stream and boundary.
    pub fn new<S>(stream: S, boundary: &str) -> Self
    where
        S: Stream<Item = std::result::Result<Bytes, std::io::Error>> + Send + 'static,
    {
        Self {
            inner: multer::Multipart::new(stream, boundary),
        }
    }

    /// Create from HTTP headers and body.
    pub fn from_request<S>(content_type: &str, body: S) -> Result<Self>
    where
        S: Stream<Item = std::result::Result<Bytes, std::io::Error>> + Send + 'static,
    {
        let boundary = multer::parse_boundary(content_type)
            .map_err(|e| StorageError::Multipart(e.to_string()))?;

        Ok(Self::new(body, &boundary))
    }

    /// Get the next field from the multipart stream.
    pub async fn next_field(&mut self) -> Result<Option<multer::Field<'static>>> {
        self.inner.next_field().await.map_err(StorageError::from)
    }

    /// Convert into a stream of fields.
    pub fn into_stream(self) -> MultipartStream {
        MultipartStream { inner: self.inner }
    }

    /// Collect all file fields into uploaded files.
    pub async fn collect_files(mut self) -> Result<Vec<UploadedFile>> {
        let mut files = Vec::new();

        while let Some(field) = self.next_field().await? {
            if field.file_name().is_some() {
                let file = UploadedFile::from_field(field).await?;
                files.push(file);
            }
        }

        Ok(files)
    }

    /// Collect all fields (both files and form data).
    pub async fn collect_all(mut self) -> Result<MultipartData> {
        let mut data = MultipartData::new();

        while let Some(field) = self.next_field().await? {
            let name = field.name().map(String::from);

            if field.file_name().is_some() {
                let file = UploadedFile::from_field(field).await?;
                if let Some(name) = name {
                    data.files.insert(name, file);
                }
            } else {
                let text = field.text().await.map_err(StorageError::from)?;
                if let Some(name) = name {
                    data.fields.insert(name, text);
                }
            }
        }

        Ok(data)
    }
}

/// Stream wrapper for multipart fields.
pub struct MultipartStream {
    inner: multer::Multipart<'static>,
}

impl MultipartStream {
    /// Get the next field.
    pub async fn next_field(&mut self) -> Result<Option<multer::Field<'static>>> {
        self.inner.next_field().await.map_err(StorageError::from)
    }
}

/// Collected multipart data.
#[derive(Debug, Default)]
pub struct MultipartData {
    /// Form fields (non-file fields).
    pub fields: std::collections::HashMap<String, String>,
    /// Uploaded files.
    pub files: std::collections::HashMap<String, UploadedFile>,
}

impl MultipartData {
    /// Create empty multipart data.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a form field value.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields.get(name).map(String::as_str)
    }

    /// Get an uploaded file.
    pub fn file(&self, name: &str) -> Option<&UploadedFile> {
        self.files.get(name)
    }

    /// Take an uploaded file (removes it from the collection).
    pub fn take_file(&mut self, name: &str) -> Option<UploadedFile> {
        self.files.remove(name)
    }

    /// Check if there are any files.
    pub fn has_files(&self) -> bool {
        !self.files.is_empty()
    }

    /// Get the number of files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Constraints for multipart parsing.
#[derive(Debug, Clone)]
pub struct MultipartConstraints {
    /// Maximum total size of all fields.
    pub max_total_size: Option<u64>,
    /// Maximum size of a single field.
    pub max_field_size: Option<u64>,
    /// Maximum number of fields.
    pub max_fields: Option<usize>,
    /// Maximum number of files.
    pub max_files: Option<usize>,
    /// Allowed field names.
    pub allowed_fields: Option<Vec<String>>,
}

impl Default for MultipartConstraints {
    fn default() -> Self {
        Self {
            max_total_size: Some(100 * 1024 * 1024), // 100 MB
            max_field_size: Some(50 * 1024 * 1024),  // 50 MB
            max_fields: Some(100),
            max_files: Some(10),
            allowed_fields: None,
        }
    }
}

impl MultipartConstraints {
    /// Create new constraints with no limits.
    pub fn unlimited() -> Self {
        Self {
            max_total_size: None,
            max_field_size: None,
            max_fields: None,
            max_files: None,
            allowed_fields: None,
        }
    }

    /// Set maximum total size.
    pub fn max_total_size(mut self, size: u64) -> Self {
        self.max_total_size = Some(size);
        self
    }

    /// Set maximum field size.
    pub fn max_field_size(mut self, size: u64) -> Self {
        self.max_field_size = Some(size);
        self
    }

    /// Set maximum number of fields.
    pub fn max_fields(mut self, count: usize) -> Self {
        self.max_fields = Some(count);
        self
    }

    /// Set maximum number of files.
    pub fn max_files(mut self, count: usize) -> Self {
        self.max_files = Some(count);
        self
    }

    /// Set allowed field names.
    pub fn allowed_fields(mut self, fields: Vec<String>) -> Self {
        self.allowed_fields = Some(fields);
        self
    }
}

/// Helper to create a Multipart from an HTTP request body.
pub fn parse_multipart<S>(content_type: &http::HeaderValue, body: S) -> Result<Multipart>
where
    S: Stream<Item = std::result::Result<Bytes, std::io::Error>> + Send + 'static,
{
    let content_type = content_type
        .to_str()
        .map_err(|_| StorageError::Multipart("Invalid content-type header".to_string()))?;

    Multipart::from_request(content_type, body)
}
