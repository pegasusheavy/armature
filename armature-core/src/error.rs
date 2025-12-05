// Error types for the Armature framework

use crate::HttpStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Route not found: {0}")]
    RouteNotFound(String),

    #[error("Method not allowed: {0}")]
    MethodNotAllowed(String),

    #[error("Dependency injection error: {0}")]
    DependencyInjection(String),

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    // 4xx Client Errors
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Payment Required: {0}")]
    PaymentRequired(String),

    #[error("Not Found: {0}")]
    NotFound(String),

    #[error("Not Acceptable: {0}")]
    NotAcceptable(String),

    #[error("Proxy Authentication Required: {0}")]
    ProxyAuthenticationRequired(String),

    #[error("Request Timeout: {0}")]
    RequestTimeout(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Gone: {0}")]
    Gone(String),

    #[error("Length Required: {0}")]
    LengthRequired(String),

    #[error("Precondition Failed: {0}")]
    PreconditionFailed(String),

    #[error("Payload Too Large: {0}")]
    PayloadTooLarge(String),

    #[error("URI Too Long: {0}")]
    UriTooLong(String),

    #[error("Unsupported Media Type: {0}")]
    UnsupportedMediaType(String),

    #[error("Range Not Satisfiable: {0}")]
    RangeNotSatisfiable(String),

    #[error("Expectation Failed: {0}")]
    ExpectationFailed(String),

    #[error("I'm a teapot: {0}")]
    ImATeapot(String),

    #[error("Misdirected Request: {0}")]
    MisdirectedRequest(String),

    #[error("Unprocessable Entity: {0}")]
    UnprocessableEntity(String),

    #[error("Locked: {0}")]
    Locked(String),

    #[error("Failed Dependency: {0}")]
    FailedDependency(String),

    #[error("Too Early: {0}")]
    TooEarly(String),

    #[error("Upgrade Required: {0}")]
    UpgradeRequired(String),

    #[error("Precondition Required: {0}")]
    PreconditionRequired(String),

    #[error("Too Many Requests: {0}")]
    TooManyRequests(String),

    #[error("Request Header Fields Too Large: {0}")]
    RequestHeaderFieldsTooLarge(String),

    #[error("Unavailable For Legal Reasons: {0}")]
    UnavailableForLegalReasons(String),

    // 5xx Server Errors
    #[error("Not Implemented: {0}")]
    NotImplemented(String),

    #[error("Bad Gateway: {0}")]
    BadGateway(String),

    #[error("Service Unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Gateway Timeout: {0}")]
    GatewayTimeout(String),

    #[error("HTTP Version Not Supported: {0}")]
    HttpVersionNotSupported(String),

    #[error("Variant Also Negotiates: {0}")]
    VariantAlsoNegotiates(String),

    #[error("Insufficient Storage: {0}")]
    InsufficientStorage(String),

    #[error("Loop Detected: {0}")]
    LoopDetected(String),

    #[error("Not Extended: {0}")]
    NotExtended(String),

    #[error("Network Authentication Required: {0}")]
    NetworkAuthenticationRequired(String),
}

impl Error {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            // Legacy mappings
            Error::RouteNotFound(_) => HttpStatus::NotFound.code(),
            Error::MethodNotAllowed(_) => HttpStatus::MethodNotAllowed.code(),
            Error::Validation(_) => HttpStatus::BadRequest.code(),
            Error::Deserialization(_) => HttpStatus::BadRequest.code(),
            Error::Forbidden(_) => HttpStatus::Forbidden.code(),

            // 4xx Client Errors
            Error::BadRequest(_) => HttpStatus::BadRequest.code(),
            Error::Unauthorized(_) => HttpStatus::Unauthorized.code(),
            Error::PaymentRequired(_) => HttpStatus::PaymentRequired.code(),
            Error::NotFound(_) => HttpStatus::NotFound.code(),
            Error::NotAcceptable(_) => HttpStatus::NotAcceptable.code(),
            Error::ProxyAuthenticationRequired(_) => HttpStatus::ProxyAuthenticationRequired.code(),
            Error::RequestTimeout(_) => HttpStatus::RequestTimeout.code(),
            Error::Conflict(_) => HttpStatus::Conflict.code(),
            Error::Gone(_) => HttpStatus::Gone.code(),
            Error::LengthRequired(_) => HttpStatus::LengthRequired.code(),
            Error::PreconditionFailed(_) => HttpStatus::PreconditionFailed.code(),
            Error::PayloadTooLarge(_) => HttpStatus::PayloadTooLarge.code(),
            Error::UriTooLong(_) => HttpStatus::UriTooLong.code(),
            Error::UnsupportedMediaType(_) => HttpStatus::UnsupportedMediaType.code(),
            Error::RangeNotSatisfiable(_) => HttpStatus::RangeNotSatisfiable.code(),
            Error::ExpectationFailed(_) => HttpStatus::ExpectationFailed.code(),
            Error::ImATeapot(_) => HttpStatus::ImATeapot.code(),
            Error::MisdirectedRequest(_) => HttpStatus::MisdirectedRequest.code(),
            Error::UnprocessableEntity(_) => HttpStatus::UnprocessableEntity.code(),
            Error::Locked(_) => HttpStatus::Locked.code(),
            Error::FailedDependency(_) => HttpStatus::FailedDependency.code(),
            Error::TooEarly(_) => HttpStatus::TooEarly.code(),
            Error::UpgradeRequired(_) => HttpStatus::UpgradeRequired.code(),
            Error::PreconditionRequired(_) => HttpStatus::PreconditionRequired.code(),
            Error::TooManyRequests(_) => HttpStatus::TooManyRequests.code(),
            Error::RequestHeaderFieldsTooLarge(_) => HttpStatus::RequestHeaderFieldsTooLarge.code(),
            Error::UnavailableForLegalReasons(_) => HttpStatus::UnavailableForLegalReasons.code(),

            // 5xx Server Errors
            Error::NotImplemented(_) => HttpStatus::NotImplemented.code(),
            Error::BadGateway(_) => HttpStatus::BadGateway.code(),
            Error::ServiceUnavailable(_) => HttpStatus::ServiceUnavailable.code(),
            Error::GatewayTimeout(_) => HttpStatus::GatewayTimeout.code(),
            Error::HttpVersionNotSupported(_) => HttpStatus::HttpVersionNotSupported.code(),
            Error::VariantAlsoNegotiates(_) => HttpStatus::VariantAlsoNegotiates.code(),
            Error::InsufficientStorage(_) => HttpStatus::InsufficientStorage.code(),
            Error::LoopDetected(_) => HttpStatus::LoopDetected.code(),
            Error::NotExtended(_) => HttpStatus::NotExtended.code(),
            Error::NetworkAuthenticationRequired(_) => {
                HttpStatus::NetworkAuthenticationRequired.code()
            }

            // Default to 500 for unmapped errors
            _ => HttpStatus::InternalServerError.code(),
        }
    }

    /// Get the HttpStatus enum for this error
    pub fn http_status(&self) -> HttpStatus {
        HttpStatus::from_code(self.status_code()).unwrap_or(HttpStatus::InternalServerError)
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.http_status().is_client_error()
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.http_status().is_server_error()
    }
}
