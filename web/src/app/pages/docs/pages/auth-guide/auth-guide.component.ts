import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-auth-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class AuthGuideComponent {
  page: DocPage = {
    title: 'Authentication & Authorization',
    subtitle: 'Complete guide to securing your Armature application with password hashing, JWT tokens, role-based access control, and guards.',
    icon: '🔐',
    badge: 'Security',
    features: [
      {
        icon: '🔑',
        title: 'Password Hashing',
        description: 'Bcrypt and Argon2 support with auto-detection'
      },
      {
        icon: '🎫',
        title: 'JWT Integration',
        description: 'Seamless token-based authentication'
      },
      {
        icon: '🛡️',
        title: 'Guards & RBAC',
        description: 'Route protection with roles and permissions'
      },
      {
        icon: '⚡',
        title: 'Pluggable Strategies',
        description: 'Local, JWT, OAuth2, and custom auth methods'
      }
    ],
    sections: [
      {
        id: 'installation',
        title: 'Installation',
        content: `<p>Add the auth feature to your <code>Cargo.toml</code>. The auth feature automatically includes JWT support.</p>`,
        codeBlocks: [
          {
            language: 'toml',
            filename: 'Cargo.toml',
            code: `[dependencies]
armature = { version = "0.1", features = ["auth"] }`
          }
        ]
      },
      {
        id: 'password-hashing',
        title: 'Password Hashing',
        content: `<p><code>armature-auth</code> supports two industry-standard password hashing algorithms:</p>
        <ul>
          <li><strong>Argon2</strong> (default, recommended) — Modern, memory-hard algorithm, winner of the Password Hashing Competition</li>
          <li><strong>Bcrypt</strong> — Battle-tested, widely compatible, good for legacy systems</li>
        </ul>`,
        subsections: [
          {
            id: 'basic-usage',
            title: 'Basic Usage',
            content: `<p>The password hasher automatically selects the best algorithm and handles all the complexity:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `use armature_auth::{PasswordHasher, PasswordVerifier};

// Default (Argon2)
let hasher = PasswordHasher::default();
let hash = hasher.hash("my-password")?;

// Verify password
let is_valid = hasher.verify("my-password", &hash)?;
assert!(is_valid);

// Use specific algorithm
use armature_auth::password::HashAlgorithm;
let bcrypt_hasher = PasswordHasher::new(HashAlgorithm::Bcrypt);
let hash = bcrypt_hasher.hash("my-password")?;`
              }
            ]
          },
          {
            id: 'auto-detection',
            title: 'Auto-Detection',
            content: `<p>The hasher automatically detects which algorithm was used from the hash format, making migrations seamless:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `let hasher = PasswordHasher::default();

// Can verify both Bcrypt and Argon2 hashes
let bcrypt_hash = "$2b$12$...";
let argon2_hash = "$argon2id$v=19$...";

hasher.verify("password", bcrypt_hash)?; // Works
hasher.verify("password", argon2_hash)?; // Also works`
              }
            ]
          }
        ]
      },
      {
        id: 'auth-service',
        title: 'Authentication Service',
        content: `<p>The <code>AuthService</code> is your central authentication component. It coordinates password hashing, JWT management, and user validation.</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use armature_auth::{AuthService, PasswordHasher};
use armature_jwt::{JwtConfig, JwtManager};

// Basic setup
let auth_service = AuthService::new();

// With JWT integration
let jwt_config = JwtConfig::new("your-secret".to_string());
let jwt_manager = JwtManager::new(jwt_config)?;
let auth_service = AuthService::with_jwt(jwt_manager);

// Custom password hasher
let hasher = PasswordHasher::new(HashAlgorithm::Bcrypt);
let auth_service = AuthService::new()
    .with_password_hasher(hasher);`
          }
        ],
        subsections: [
          {
            id: 'service-methods',
            title: 'Service Methods',
            content: `<p>The AuthService provides all the methods you need for authentication:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `// Hash a password
let hash = auth_service.hash_password("password")?;

// Verify a password
let is_valid = auth_service.verify_password("password", &hash)?;

// Validate a user
auth_service.validate(&user)?;

// Access JWT manager
if let Some(jwt) = auth_service.jwt_manager() {
    let token = jwt.sign(&claims)?;
}`
              }
            ]
          }
        ]
      },
      {
        id: 'guards',
        title: 'Guards',
        content: `<p>Guards protect routes by enforcing authentication and authorization rules before the handler runs.</p>`,
        subsections: [
          {
            id: 'auth-guard',
            title: 'Authentication Guard',
            content: `<p>Ensures the request has a valid authentication token:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `use armature_auth::AuthGuard;

let guard = AuthGuard::new();

// Check if request can proceed
if guard.can_activate(&request).await? {
    // Request is authenticated
}`
              }
            ]
          },
          {
            id: 'role-guard',
            title: 'Role Guard',
            content: `<p>Requires specific roles for access:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `use armature_auth::RoleGuard;

// Require ANY of these roles
let guard = RoleGuard::any(vec!["admin".into(), "moderator".into()]);

// Require ALL of these roles
let guard = RoleGuard::all(vec!["admin".into(), "verified".into()]);

// Check user roles
let has_access = guard.check_roles(&user);`
              }
            ]
          },
          {
            id: 'permission-guard',
            title: 'Permission Guard',
            content: `<p>Fine-grained permission checking:</p>`,
            codeBlocks: [
              {
                language: 'rust',
                code: `use armature_auth::PermissionGuard;

// Require ANY of these permissions
let guard = PermissionGuard::any(vec![
    "posts:read".to_string(),
    "posts:list".to_string()
]);

// Require ALL of these permissions
let guard = PermissionGuard::all(vec![
    "posts:read".to_string(),
    "posts:write".to_string()
]);`
              }
            ]
          }
        ]
      },
      {
        id: 'complete-example',
        title: 'Complete Example',
        content: `<p>Here's a full authentication flow with registration, login, and protected routes:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'auth_controller.rs',
            code: `use armature_auth::{AuthService, AuthUser, UserContext};
use armature_jwt::{Claims, JwtConfig, JwtManager};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct RegisterRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    user: UserInfo,
}

async fn register(
    auth_service: &AuthService,
    req: RegisterRequest,
) -> Result<User, Error> {
    // Hash password
    let password_hash = auth_service.hash_password(&req.password)?;

    // Create user
    let user = User {
        id: generate_id(),
        email: req.email,
        password_hash,
        roles: vec!["user".to_string()],
        active: true,
    };

    // Save to database
    save_user(&user).await?;
    Ok(user)
}

async fn login(
    auth_service: &AuthService,
    req: LoginRequest,
) -> Result<AuthResponse, Error> {
    // Find user
    let user = find_user_by_email(&req.email).await?;

    // Verify password
    if !auth_service.verify_password(&req.password, &user.password_hash)? {
        return Err(Error::InvalidCredentials);
    }

    // Generate tokens
    let jwt_manager = auth_service.jwt_manager().unwrap();
    let claims = Claims::new(UserContext::new(user.id.clone())
        .with_email(user.email.clone())
        .with_roles(user.roles.clone()))
        .with_expiration(3600);

    let token_pair = jwt_manager.generate_token_pair(&claims)?;

    Ok(AuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        user: UserInfo { id: user.id, email: user.email, roles: user.roles },
    })
}`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Use Argon2</strong> — It's the default and recommended algorithm for new applications</li>
          <li><strong>Short-lived access tokens</strong> — Use 15-minute expiration for access tokens</li>
          <li><strong>Secure secrets</strong> — Store JWT secrets in environment variables, never in code</li>
          <li><strong>Validate users</strong> — Always check if users are active after authentication</li>
          <li><strong>Rate limit auth endpoints</strong> — Prevent brute-force attacks</li>
          <li><strong>Never store plain passwords</strong> — Always hash before storing</li>
        </ul>`
      }
    ],
    relatedDocs: [
      {
        id: 'oauth2-providers',
        title: 'OAuth2 & Social Login',
        description: 'Add Google, GitHub, Microsoft and other social login providers'
      },
      {
        id: 'security-guide',
        title: 'Security Best Practices',
        description: 'CORS, CSRF, headers, and more security features'
      },
      {
        id: 'guards-interceptors',
        title: 'Guards & Interceptors',
        description: 'Route protection and request/response transformations'
      }
    ],
    seeAlso: [
      { title: 'Session Management', id: 'session-guide' },
      { title: 'Rate Limiting', id: 'rate-limiting' },
      { title: 'Configuration Guide', id: 'config-guide' }
    ]
  };
}

