//! Common utilities for examples

use std::net::TcpListener;

/// Find an available port on the local machine.
///
/// This is useful for examples and tests that need to bind to a port
/// without conflicting with other services.
pub fn find_available_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port")
        .local_addr()
        .expect("Failed to get local address")
        .port()
}

/// Default port to use if find_available_port fails or for documentation.
pub const DEFAULT_PORT: u16 = 3000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_available_port() {
        let port = find_available_port();
        assert!(port > 0);
        assert!(port < 65535);
    }
}
