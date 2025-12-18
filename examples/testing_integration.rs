//! Integration Testing Example
//!
//! Demonstrates database setup/teardown for integration tests.

use armature_testing::integration::*;
use async_trait::async_trait;
use std::sync::Arc;

// Example database helper implementation
struct PostgresTestHelper {
    connection_string: String,
}

impl PostgresTestHelper {
    fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }
}

#[async_trait]
impl DatabaseTestHelper for PostgresTestHelper {
    async fn setup(&self) -> Result<(), IntegrationTestError> {
        println!("Setting up database: {}", self.connection_string);

        // In a real implementation, you would:
        // 1. Connect to the database
        // 2. Run migrations
        // 3. Seed test data

        // Example SQL (pseudo-code):
        // CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255));
        // INSERT INTO users (name) VALUES ('Alice'), ('Bob');

        println!("✅ Database setup complete");
        Ok(())
    }

    async fn teardown(&self) -> Result<(), IntegrationTestError> {
        println!("Tearing down database: {}", self.connection_string);

        // In a real implementation:
        // 1. Drop tables
        // 2. Clean up test data

        // Example SQL (pseudo-code):
        // DROP TABLE IF EXISTS users CASCADE;

        println!("✅ Database teardown complete");
        Ok(())
    }

    async fn migrate(&self) -> Result<(), IntegrationTestError> {
        println!("Running migrations...");
        // Run database migrations
        Ok(())
    }

    async fn seed(&self) -> Result<(), IntegrationTestError> {
        println!("Seeding test data...");
        // Insert test data
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Integration Testing Example ===\n");

    // 1. Basic usage with automatic setup/teardown
    println!("1. Basic fixture usage:");
    let helper = Arc::new(PostgresTestHelper::new("postgres://localhost/test_db"));
    let fixture = TestFixture::new(helper.clone());

    fixture
        .run_test(|| async {
            println!("   Running test...");
            // Your test code here
            // Database is automatically set up and torn down
            println!("   ✅ Test passed");
            Ok(())
        })
        .await?;

    println!();

    // 2. Manual setup/teardown control
    println!("2. Manual control:");
    let fixture = TestFixture::new(helper.clone()).without_auto_cleanup();

    fixture.setup().await?;
    println!("   Running test with manual control...");
    // Your test code
    println!("   ✅ Test passed");
    fixture.teardown().await?;

    println!();

    // 3. Database seeder
    println!("3. Database seeder:");
    let seeder = DatabaseSeeder::new()
        .add_fixture("users")
        .add_fixture("posts")
        .add_fixture("comments");

    println!("   Fixtures to load:");
    for fixture_name in seeder.fixtures() {
        println!("     - {}", fixture_name);
    }

    println!();

    // 4. Integration test builder with hooks
    println!("4. Integration test builder:");
    let _test_suite = IntegrationTestBuilder::new("User API Tests")
        .before_each(|| async {
            println!("   Before each test...");
        })
        .after_each(|| async {
            println!("   After each test...");
        });

    println!("   ✅ Test suite configured");

    println!();
    println!("=== Integration Testing Complete ===\n");

    Ok(())
}
