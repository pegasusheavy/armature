// Testing utilities for Armature framework

mod assertions;
mod mock;
mod test_app;
mod test_client;
mod test_container;

pub use assertions::{assert_header, assert_json, assert_status};
pub use mock::{MockController, MockProvider, MockService};
pub use test_app::{TestApp, TestAppBuilder};
pub use test_client::{TestClient, TestResponse};
pub use test_container::TestContainer;

// Re-export common testing utilities
pub use tokio::test as tokio_test;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_exports() {
        // Ensure module compiles
    }
}
