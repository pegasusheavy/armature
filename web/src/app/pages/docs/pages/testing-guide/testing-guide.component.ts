import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-testing-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class TestingGuideComponent {
  page: DocPage = {
    title: 'Testing Guide',
    subtitle: 'Write comprehensive tests for your Armature application with unit, integration, and end-to-end testing strategies.',
    icon: '🧪',
    badge: 'Testing',
    features: [
      { icon: '✅', title: 'Unit Tests', description: 'Test individual components' },
      { icon: '🔗', title: 'Integration Tests', description: 'Test service interactions' },
      { icon: '🌐', title: 'E2E Tests', description: 'Full HTTP request testing' },
      { icon: '🐳', title: 'Test Containers', description: 'Real database testing' }
    ],
    sections: [
      {
        id: 'unit-testing',
        title: 'Unit Testing',
        content: `<p>Test individual services and functions:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tests/unit.rs',
            code: `use my_app::services::UserService;

#[test]
fn test_validate_email() {
    assert!(UserService::validate_email("user@example.com"));
    assert!(!UserService::validate_email("invalid-email"));
    assert!(!UserService::validate_email(""));
}

#[test]
fn test_hash_password() {
    let hash = UserService::hash_password("secret123").unwrap();
    assert!(UserService::verify_password("secret123", &hash));
    assert!(!UserService::verify_password("wrong", &hash));
}

#[tokio::test]
async fn test_async_function() {
    let service = UserService::default();
    let result = service.process_data("input").await;
    assert_eq!(result, "expected_output");
}`
          }
        ]
      },
      {
        id: 'integration-testing',
        title: 'Integration Testing',
        content: `<p>Test services with their dependencies:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tests/integration.rs',
            code: `use armature_testing::*;

#[tokio::test]
async fn test_user_creation_flow() {
    // Setup test container with real Postgres
    let db = TestContainer::postgres().await;
    let redis = TestContainer::redis().await;

    // Create test application
    let app = TestApp::builder()
        .with_database(&db.connection_string())
        .with_redis(&redis.connection_string())
        .build()
        .await;

    // Create user via service
    let user_service = app.get::<UserService>();
    let user = user_service.create(CreateUser {
        email: "test@example.com".into(),
        password: "secret123".into(),
    }).await.unwrap();

    // Verify user exists
    let found = user_service.find_by_email("test@example.com").await.unwrap();
    assert_eq!(found.id, user.id);
}`
          }
        ]
      },
      {
        id: 'http-testing',
        title: 'HTTP/E2E Testing',
        content: `<p>Test full HTTP request/response cycles:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'tests/e2e.rs',
            code: `use armature_testing::*;

#[tokio::test]
async fn test_api_endpoints() {
    let app = TestApp::new().await;

    // Test GET endpoint
    let response = app.get("/api/health").await;
    assert_eq!(response.status(), 200);

    // Test POST with JSON body
    let response = app.post("/api/users")
        .json(&json!({
            "email": "test@example.com",
            "password": "secret123"
        }))
        .await;
    assert_eq!(response.status(), 201);

    let user: User = response.json().await;
    assert_eq!(user.email, "test@example.com");

    // Test authenticated endpoint
    let token = app.login("test@example.com", "secret123").await;
    let response = app.get("/api/profile")
        .bearer_auth(&token)
        .await;
    assert_eq!(response.status(), 200);
}`
          }
        ]
      },
      {
        id: 'test-containers',
        title: 'Test Containers',
        content: `<p>Spin up real databases for testing:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_testing::TestContainer;

#[tokio::test]
async fn test_with_real_postgres() {
    // Starts a real Postgres container
    let postgres = TestContainer::postgres()
        .with_database("test_db")
        .await;

    // Run migrations
    postgres.migrate("./migrations").await;

    // Use the connection
    let pool = postgres.pool().await;
    let result = sqlx::query("SELECT 1 as num")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Container is automatically cleaned up
}

#[tokio::test]
async fn test_with_redis() {
    let redis = TestContainer::redis().await;

    let client = redis.client().await;
    client.set("key", "value").await.unwrap();

    let value: String = client.get("key").await.unwrap();
    assert_eq!(value, "value");
}`
          }
        ]
      },
      {
        id: 'mocking',
        title: 'Mocking Services',
        content: `<p>Replace services with mocks for isolated testing:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use mockall::predicate::*;
use armature_testing::*;

// Define mock
mock! {
    pub EmailService {}
    #[async_trait]
    impl EmailSender for EmailService {
        async fn send(&self, to: &str, subject: &str, body: &str) -> Result<()>;
    }
}

#[tokio::test]
async fn test_with_mock_email() {
    let mut mock_email = MockEmailService::new();

    // Set expectations
    mock_email
        .expect_send()
        .with(eq("user@example.com"), any(), any())
        .times(1)
        .returning(|_, _, _| Ok(()));

    // Create app with mock
    let app = TestApp::builder()
        .mock::<dyn EmailSender>(mock_email)
        .build()
        .await;

    // Test endpoint that sends email
    let response = app.post("/api/register")
        .json(&json!({ "email": "user@example.com" }))
        .await;

    assert_eq!(response.status(), 201);
    // Mock expectations are verified on drop
}`
          }
        ]
      },
      {
        id: 'test-fixtures',
        title: 'Test Fixtures',
        content: `<p>Reuse test data across tests:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_testing::*;

// Define fixtures
#[fixture]
async fn test_user(app: &TestApp) -> User {
    app.get::<UserService>()
        .create(CreateUser {
            email: "fixture@example.com".into(),
            password: "password".into(),
        })
        .await
        .unwrap()
}

#[fixture]
async fn admin_user(app: &TestApp) -> User {
    let mut user = test_user(app).await;
    user.role = "admin".into();
    app.get::<UserService>().update(&user).await.unwrap()
}

#[tokio::test]
async fn test_admin_endpoint(admin_user: User) {
    let app = TestApp::new().await;
    let token = app.token_for(&admin_user);

    let response = app.get("/api/admin/dashboard")
        .bearer_auth(&token)
        .await;

    assert_eq!(response.status(), 200);
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Test behavior, not implementation</strong> — Focus on inputs and outputs</li>
          <li><strong>Use real databases when possible</strong> — Test containers are fast enough</li>
          <li><strong>Isolate tests</strong> — Each test should be independent</li>
          <li><strong>Name tests descriptively</strong> — <code>test_login_with_invalid_password_returns_401</code></li>
          <li><strong>Test error cases</strong> — Happy path is just the start</li>
          <li><strong>Keep tests fast</strong> — Slow tests get skipped</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'di-guide', title: 'Dependency Injection', description: 'Mock injection patterns' },
      { id: 'testing-coverage', title: 'Coverage', description: 'Code coverage reporting' }
    ],
    seeAlso: [
      { title: 'Health Checks', id: 'health-check' },
      { title: 'Logging', id: 'logging-guide' }
    ]
  };
}

