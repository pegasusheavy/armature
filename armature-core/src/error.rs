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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_error_status() {
        let err = Error::Internal("test".to_string());
        assert_eq!(err.status_code(), 500);
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_not_found_error() {
        let err = Error::NotFound("resource".to_string());
        assert_eq!(err.status_code(), 404);
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_unauthorized_error() {
        let err = Error::Unauthorized("auth required".to_string());
        assert_eq!(err.status_code(), 401);
        assert!(err.is_client_error());
    }

    #[test]
    fn test_forbidden_error() {
        let err = Error::Forbidden("access denied".to_string());
        assert_eq!(err.status_code(), 403);
        assert!(err.is_client_error());
    }

    #[test]
    fn test_bad_request_error() {
        let err = Error::BadRequest("invalid input".to_string());
        assert_eq!(err.status_code(), 400);
    }

    #[test]
    fn test_conflict_error() {
        let err = Error::Conflict("resource conflict".to_string());
        assert_eq!(err.status_code(), 409);
    }

    #[test]
    fn test_gone_error() {
        let err = Error::Gone("resource deleted".to_string());
        assert_eq!(err.status_code(), 410);
    }

    #[test]
    fn test_payload_too_large() {
        let err = Error::PayloadTooLarge("file too big".to_string());
        assert_eq!(err.status_code(), 413);
    }

    #[test]
    fn test_unsupported_media_type() {
        let err = Error::UnsupportedMediaType("invalid content-type".to_string());
        assert_eq!(err.status_code(), 415);
    }

    #[test]
    fn test_too_many_requests() {
        let err = Error::TooManyRequests("rate limited".to_string());
        assert_eq!(err.status_code(), 429);
    }

    #[test]
    fn test_not_implemented() {
        let err = Error::NotImplemented("feature not ready".to_string());
        assert_eq!(err.status_code(), 501);
        assert!(err.is_server_error());
    }

    #[test]
    fn test_bad_gateway() {
        let err = Error::BadGateway("upstream error".to_string());
        assert_eq!(err.status_code(), 502);
    }

    #[test]
    fn test_service_unavailable() {
        let err = Error::ServiceUnavailable("maintenance".to_string());
        assert_eq!(err.status_code(), 503);
    }

    #[test]
    fn test_gateway_timeout() {
        let err = Error::GatewayTimeout("upstream timeout".to_string());
        assert_eq!(err.status_code(), 504);
    }

    #[test]
    fn test_method_not_allowed() {
        let err = Error::MethodNotAllowed("POST not allowed".to_string());
        assert_eq!(err.status_code(), 405);
    }

    #[test]
    fn test_not_acceptable() {
        let err = Error::NotAcceptable("format not supported".to_string());
        assert_eq!(err.status_code(), 406);
    }

    #[test]
    fn test_request_timeout() {
        let err = Error::RequestTimeout("request took too long".to_string());
        assert_eq!(err.status_code(), 408);
    }

    #[test]
    fn test_unprocessable_entity() {
        let err = Error::UnprocessableEntity("validation failed".to_string());
        assert_eq!(err.status_code(), 422);
    }

    #[test]
    fn test_locked() {
        let err = Error::Locked("resource locked".to_string());
        assert_eq!(err.status_code(), 423);
    }

    #[test]
    fn test_upgrade_required() {
        let err = Error::UpgradeRequired("http/2 required".to_string());
        assert_eq!(err.status_code(), 426);
    }

    #[test]
    fn test_precondition_required() {
        let err = Error::PreconditionRequired("if-match required".to_string());
        assert_eq!(err.status_code(), 428);
    }

    #[test]
    fn test_http_status_conversion() {
        let err = Error::NotFound("test".to_string());
        let status = err.http_status();
        assert_eq!(status, HttpStatus::NotFound);
    }

    #[test]
    fn test_error_display() {
        let err = Error::Internal("something went wrong".to_string());
        let display = format!("{}", err);
        assert!(display.contains("something went wrong"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }

    #[test]
    fn test_serialization_error() {
        let err = Error::Serialization("failed to serialize".to_string());
        assert!(format!("{}", err).contains("Serialization"));
    }

    #[test]
    fn test_deserialization_error() {
        let err = Error::Deserialization("failed to deserialize".to_string());
        assert!(format!("{}", err).contains("Deserialization"));
    }

    #[test]
    fn test_validation_error() {
        let err = Error::Validation("validation failed".to_string());
        assert!(format!("{}", err).contains("Validation"));
    }

    #[test]
    fn test_http_error() {
        let err = Error::Http("http error".to_string());
        assert!(format!("{}", err).contains("HTTP error"));
    }

    #[test]
    fn test_route_not_found_error() {
        let err = Error::RouteNotFound("/api/users".to_string());
        assert!(format!("{}", err).contains("Route not found"));
    }

    #[test]
    fn test_im_a_teapot() {
        let err = Error::ImATeapot("I'm a teapot".to_string());
        assert_eq!(err.status_code(), 418);
    }

    #[test]
    fn test_misdirected_request() {
        let err = Error::MisdirectedRequest("wrong server".to_string());
        assert_eq!(err.status_code(), 421);
    }

    #[test]
    fn test_failed_dependency() {
        let err = Error::FailedDependency("dependent request failed".to_string());
        assert_eq!(err.status_code(), 424);
    }

    #[test]
    fn test_too_early() {
        let err = Error::TooEarly("request too early".to_string());
        assert_eq!(err.status_code(), 425);
    }

    #[test]
    fn test_request_header_fields_too_large() {
        let err = Error::RequestHeaderFieldsTooLarge("headers too big".to_string());
        assert_eq!(err.status_code(), 431);
    }

    #[test]
    fn test_unavailable_for_legal_reasons() {
        let err = Error::UnavailableForLegalReasons("blocked by law".to_string());
        assert_eq!(err.status_code(), 451);
    }

    #[test]
    fn test_http_version_not_supported() {
        let err = Error::HttpVersionNotSupported("http/0.9 not supported".to_string());
        assert_eq!(err.status_code(), 505);
    }

    #[test]
    fn test_variant_also_negotiates() {
        let err = Error::VariantAlsoNegotiates("circular reference".to_string());
        assert_eq!(err.status_code(), 506);
    }

    #[test]
    fn test_insufficient_storage() {
        let err = Error::InsufficientStorage("disk full".to_string());
        assert_eq!(err.status_code(), 507);
    }

    #[test]
    fn test_loop_detected() {
        let err = Error::LoopDetected("infinite loop".to_string());
        assert_eq!(err.status_code(), 508);
    }

    #[test]
    fn test_not_extended() {
        let err = Error::NotExtended("extension required".to_string());
        assert_eq!(err.status_code(), 510);
    }

    #[test]
    fn test_network_authentication_required() {
        let err = Error::NetworkAuthenticationRequired("proxy auth required".to_string());
        assert_eq!(err.status_code(), 511);
    }

    #[test]
    fn test_length_required() {
        let err = Error::LengthRequired("content-length missing".to_string());
        assert_eq!(err.status_code(), 411);
    }

    #[test]
    fn test_precondition_failed() {
        let err = Error::PreconditionFailed("if-match failed".to_string());
        assert_eq!(err.status_code(), 412);
    }

    #[test]
    fn test_uri_too_long() {
        let err = Error::UriTooLong("url too long".to_string());
        assert_eq!(err.status_code(), 414);
    }

    #[test]
    fn test_range_not_satisfiable() {
        let err = Error::RangeNotSatisfiable("invalid range".to_string());
        assert_eq!(err.status_code(), 416);
    }

    #[test]
    fn test_expectation_failed() {
        let err = Error::ExpectationFailed("expect header failed".to_string());
        assert_eq!(err.status_code(), 417);
    }

    #[test]
    fn test_proxy_authentication_required() {
        let err = Error::ProxyAuthenticationRequired("proxy auth needed".to_string());
        assert_eq!(err.status_code(), 407);
    }

    #[test]
    fn test_client_error_range() {
        for code in 400..500 {
            if let Some(_status) = HttpStatus::from_code(code) {
                let err = Error::BadRequest("test".to_string());
                if err.status_code() == code {
                    assert!(err.is_client_error());
                    assert!(!err.is_server_error());
                }
            }
        }
    }

    #[test]
    fn test_server_error_range() {
        for code in 500..600 {
            if let Some(_status) = HttpStatus::from_code(code) {
                let err = Error::Internal("test".to_string());
                if err.status_code() == code {
                    assert!(err.is_server_error());
                    assert!(!err.is_client_error());
                }
            }
        }
    }
}
