// HTTP Status Codes

/// HTTP status codes as defined in RFC 7231, RFC 6585, and additional standards
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    // 1xx Informational
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,
    EarlyHints = 103,

    // 2xx Success
    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,

    // 3xx Redirection
    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    // 4xx Client Errors
    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    // 5xx Server Errors
    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl HttpStatus {
    /// Get the numeric status code
    pub fn code(&self) -> u16 {
        *self as u16
    }

    /// Get the reason phrase for the status code
    pub fn reason(&self) -> &'static str {
        match self {
            // 1xx
            HttpStatus::Continue => "Continue",
            HttpStatus::SwitchingProtocols => "Switching Protocols",
            HttpStatus::Processing => "Processing",
            HttpStatus::EarlyHints => "Early Hints",

            // 2xx
            HttpStatus::Ok => "OK",
            HttpStatus::Created => "Created",
            HttpStatus::Accepted => "Accepted",
            HttpStatus::NonAuthoritativeInformation => "Non-Authoritative Information",
            HttpStatus::NoContent => "No Content",
            HttpStatus::ResetContent => "Reset Content",
            HttpStatus::PartialContent => "Partial Content",
            HttpStatus::MultiStatus => "Multi-Status",
            HttpStatus::AlreadyReported => "Already Reported",
            HttpStatus::ImUsed => "IM Used",

            // 3xx
            HttpStatus::MultipleChoices => "Multiple Choices",
            HttpStatus::MovedPermanently => "Moved Permanently",
            HttpStatus::Found => "Found",
            HttpStatus::SeeOther => "See Other",
            HttpStatus::NotModified => "Not Modified",
            HttpStatus::UseProxy => "Use Proxy",
            HttpStatus::TemporaryRedirect => "Temporary Redirect",
            HttpStatus::PermanentRedirect => "Permanent Redirect",

            // 4xx
            HttpStatus::BadRequest => "Bad Request",
            HttpStatus::Unauthorized => "Unauthorized",
            HttpStatus::PaymentRequired => "Payment Required",
            HttpStatus::Forbidden => "Forbidden",
            HttpStatus::NotFound => "Not Found",
            HttpStatus::MethodNotAllowed => "Method Not Allowed",
            HttpStatus::NotAcceptable => "Not Acceptable",
            HttpStatus::ProxyAuthenticationRequired => "Proxy Authentication Required",
            HttpStatus::RequestTimeout => "Request Timeout",
            HttpStatus::Conflict => "Conflict",
            HttpStatus::Gone => "Gone",
            HttpStatus::LengthRequired => "Length Required",
            HttpStatus::PreconditionFailed => "Precondition Failed",
            HttpStatus::PayloadTooLarge => "Payload Too Large",
            HttpStatus::UriTooLong => "URI Too Long",
            HttpStatus::UnsupportedMediaType => "Unsupported Media Type",
            HttpStatus::RangeNotSatisfiable => "Range Not Satisfiable",
            HttpStatus::ExpectationFailed => "Expectation Failed",
            HttpStatus::ImATeapot => "I'm a teapot",
            HttpStatus::MisdirectedRequest => "Misdirected Request",
            HttpStatus::UnprocessableEntity => "Unprocessable Entity",
            HttpStatus::Locked => "Locked",
            HttpStatus::FailedDependency => "Failed Dependency",
            HttpStatus::TooEarly => "Too Early",
            HttpStatus::UpgradeRequired => "Upgrade Required",
            HttpStatus::PreconditionRequired => "Precondition Required",
            HttpStatus::TooManyRequests => "Too Many Requests",
            HttpStatus::RequestHeaderFieldsTooLarge => "Request Header Fields Too Large",
            HttpStatus::UnavailableForLegalReasons => "Unavailable For Legal Reasons",

            // 5xx
            HttpStatus::InternalServerError => "Internal Server Error",
            HttpStatus::NotImplemented => "Not Implemented",
            HttpStatus::BadGateway => "Bad Gateway",
            HttpStatus::ServiceUnavailable => "Service Unavailable",
            HttpStatus::GatewayTimeout => "Gateway Timeout",
            HttpStatus::HttpVersionNotSupported => "HTTP Version Not Supported",
            HttpStatus::VariantAlsoNegotiates => "Variant Also Negotiates",
            HttpStatus::InsufficientStorage => "Insufficient Storage",
            HttpStatus::LoopDetected => "Loop Detected",
            HttpStatus::NotExtended => "Not Extended",
            HttpStatus::NetworkAuthenticationRequired => "Network Authentication Required",
        }
    }

    /// Check if status is informational (1xx)
    pub fn is_informational(&self) -> bool {
        (100..200).contains(&self.code())
    }

    /// Check if status is successful (2xx)
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.code())
    }

    /// Check if status is redirection (3xx)
    pub fn is_redirection(&self) -> bool {
        (300..400).contains(&self.code())
    }

    /// Check if status is client error (4xx)
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.code())
    }

    /// Check if status is server error (5xx)
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.code())
    }

    /// Check if status is an error (4xx or 5xx)
    pub fn is_error(&self) -> bool {
        self.is_client_error() || self.is_server_error()
    }

    /// Create status from u16 code
    pub fn from_code(code: u16) -> Option<Self> {
        match code {
            // 1xx
            100 => Some(HttpStatus::Continue),
            101 => Some(HttpStatus::SwitchingProtocols),
            102 => Some(HttpStatus::Processing),
            103 => Some(HttpStatus::EarlyHints),

            // 2xx
            200 => Some(HttpStatus::Ok),
            201 => Some(HttpStatus::Created),
            202 => Some(HttpStatus::Accepted),
            203 => Some(HttpStatus::NonAuthoritativeInformation),
            204 => Some(HttpStatus::NoContent),
            205 => Some(HttpStatus::ResetContent),
            206 => Some(HttpStatus::PartialContent),
            207 => Some(HttpStatus::MultiStatus),
            208 => Some(HttpStatus::AlreadyReported),
            226 => Some(HttpStatus::ImUsed),

            // 3xx
            300 => Some(HttpStatus::MultipleChoices),
            301 => Some(HttpStatus::MovedPermanently),
            302 => Some(HttpStatus::Found),
            303 => Some(HttpStatus::SeeOther),
            304 => Some(HttpStatus::NotModified),
            305 => Some(HttpStatus::UseProxy),
            307 => Some(HttpStatus::TemporaryRedirect),
            308 => Some(HttpStatus::PermanentRedirect),

            // 4xx
            400 => Some(HttpStatus::BadRequest),
            401 => Some(HttpStatus::Unauthorized),
            402 => Some(HttpStatus::PaymentRequired),
            403 => Some(HttpStatus::Forbidden),
            404 => Some(HttpStatus::NotFound),
            405 => Some(HttpStatus::MethodNotAllowed),
            406 => Some(HttpStatus::NotAcceptable),
            407 => Some(HttpStatus::ProxyAuthenticationRequired),
            408 => Some(HttpStatus::RequestTimeout),
            409 => Some(HttpStatus::Conflict),
            410 => Some(HttpStatus::Gone),
            411 => Some(HttpStatus::LengthRequired),
            412 => Some(HttpStatus::PreconditionFailed),
            413 => Some(HttpStatus::PayloadTooLarge),
            414 => Some(HttpStatus::UriTooLong),
            415 => Some(HttpStatus::UnsupportedMediaType),
            416 => Some(HttpStatus::RangeNotSatisfiable),
            417 => Some(HttpStatus::ExpectationFailed),
            418 => Some(HttpStatus::ImATeapot),
            421 => Some(HttpStatus::MisdirectedRequest),
            422 => Some(HttpStatus::UnprocessableEntity),
            423 => Some(HttpStatus::Locked),
            424 => Some(HttpStatus::FailedDependency),
            425 => Some(HttpStatus::TooEarly),
            426 => Some(HttpStatus::UpgradeRequired),
            428 => Some(HttpStatus::PreconditionRequired),
            429 => Some(HttpStatus::TooManyRequests),
            431 => Some(HttpStatus::RequestHeaderFieldsTooLarge),
            451 => Some(HttpStatus::UnavailableForLegalReasons),

            // 5xx
            500 => Some(HttpStatus::InternalServerError),
            501 => Some(HttpStatus::NotImplemented),
            502 => Some(HttpStatus::BadGateway),
            503 => Some(HttpStatus::ServiceUnavailable),
            504 => Some(HttpStatus::GatewayTimeout),
            505 => Some(HttpStatus::HttpVersionNotSupported),
            506 => Some(HttpStatus::VariantAlsoNegotiates),
            507 => Some(HttpStatus::InsufficientStorage),
            508 => Some(HttpStatus::LoopDetected),
            510 => Some(HttpStatus::NotExtended),
            511 => Some(HttpStatus::NetworkAuthenticationRequired),

            _ => None,
        }
    }
}

impl std::fmt::Display for HttpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.code(), self.reason())
    }
}

impl From<HttpStatus> for u16 {
    fn from(status: HttpStatus) -> Self {
        status.code()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_code() {
        assert_eq!(HttpStatus::Ok.code(), 200);
        assert_eq!(HttpStatus::NotFound.code(), 404);
        assert_eq!(HttpStatus::InternalServerError.code(), 500);
    }

    #[test]
    fn test_status_reason() {
        assert_eq!(HttpStatus::Ok.reason(), "OK");
        assert_eq!(HttpStatus::NotFound.reason(), "Not Found");
        assert_eq!(
            HttpStatus::InternalServerError.reason(),
            "Internal Server Error"
        );
    }

    #[test]
    fn test_status_categories() {
        assert!(HttpStatus::Ok.is_success());
        assert!(HttpStatus::NotFound.is_client_error());
        assert!(HttpStatus::InternalServerError.is_server_error());
        assert!(HttpStatus::NotFound.is_error());
        assert!(!HttpStatus::Ok.is_error());
    }

    #[test]
    fn test_from_code() {
        assert_eq!(HttpStatus::from_code(200), Some(HttpStatus::Ok));
        assert_eq!(HttpStatus::from_code(404), Some(HttpStatus::NotFound));
        assert_eq!(HttpStatus::from_code(999), None);
    }

    #[test]
    fn test_display() {
        assert_eq!(HttpStatus::Ok.to_string(), "200 OK");
        assert_eq!(HttpStatus::NotFound.to_string(), "404 Not Found");
    }
}
