#![allow(dead_code)]
// OAuth2 Providers example - demonstrating all supported providers

use armature_auth::OAuth2Provider;
use armature_auth::providers::{
    Auth0Config, Auth0Provider, AwsCognitoConfig, AwsCognitoProvider, GoogleConfig, GoogleProvider,
    MicrosoftEntraConfig, MicrosoftEntraProvider, OktaConfig, OktaProvider,
};

/// Read a credential from the environment. Client secrets are never
/// hardcoded; without the variable a visibly fake placeholder is used, which
/// is enough to build authorization URLs but can never complete a login.
fn env_or_placeholder(name: &str, placeholder: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        println!("   (set {name} to use real credentials)");
        placeholder.to_string()
    })
}

#[tokio::main]
async fn main() {
    println!("🔐 Armature OAuth2 Providers Example");
    println!("====================================\n");

    // 1. Google OAuth2
    println!("1. Google OAuth2 Provider");
    println!("   Configuration:");
    let google_config = GoogleConfig::new(
        env_or_placeholder(
            "GOOGLE_CLIENT_ID",
            "your-client-id.apps.googleusercontent.com",
        ),
        env_or_placeholder("GOOGLE_CLIENT_SECRET", "your-client-secret"),
        "http://localhost:3000/auth/google/callback".to_string(),
    );

    match GoogleProvider::new(google_config) {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());

            if let Ok((auth_url, csrf_token)) = provider.authorization_url() {
                println!(
                    "   ✓ Auth URL: {}...",
                    &auth_url.as_str()[..80.min(auth_url.as_str().len())]
                );
                println!(
                    "   ✓ State (store the PendingAuth server-side, keyed by this): {}",
                    csrf_token.state()
                );
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // 2. Microsoft Entra (Azure AD)
    println!("2. Microsoft Entra Provider");
    println!("   Configuration:");
    let microsoft_config = MicrosoftEntraConfig::common(
        env_or_placeholder("MICROSOFT_CLIENT_ID", "your-application-id"),
        env_or_placeholder("MICROSOFT_CLIENT_SECRET", "your-client-secret"),
        "http://localhost:3000/auth/microsoft/callback".to_string(),
    );

    match MicrosoftEntraProvider::new(microsoft_config) {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());

            if let Ok((auth_url, _)) = provider.authorization_url() {
                println!(
                    "   ✓ Auth URL: {}...",
                    &auth_url.as_str()[..80.min(auth_url.as_str().len())]
                );
            }

            println!("   ℹ Tenant types:");
            println!("     - common: All Azure AD accounts");
            println!("     - organizations: Work/school accounts only");
            println!("     - consumers: Personal Microsoft accounts");
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // 3. AWS Cognito
    println!("3. AWS Cognito Provider");
    println!("   Configuration:");
    let cognito_config = AwsCognitoConfig::new(
        env_or_placeholder("COGNITO_CLIENT_ID", "your-client-id"),
        env_or_placeholder("COGNITO_CLIENT_SECRET", "your-client-secret"),
        "http://localhost:3000/auth/cognito/callback".to_string(),
        env_or_placeholder(
            "COGNITO_DOMAIN",
            "your-app.auth.us-east-1.amazoncognito.com",
        ),
        "us-east-1".to_string(),
    );

    match AwsCognitoProvider::new(cognito_config) {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());

            if let Ok((auth_url, _)) = provider.authorization_url() {
                println!(
                    "   ✓ Auth URL: {}...",
                    &auth_url.as_str()[..80.min(auth_url.as_str().len())]
                );
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // 4. Okta
    println!("4. Okta Provider");
    println!("   Configuration:");
    let okta_config = OktaConfig::new(
        env_or_placeholder("OKTA_CLIENT_ID", "your-client-id"),
        env_or_placeholder("OKTA_CLIENT_SECRET", "your-client-secret"),
        "http://localhost:3000/auth/okta/callback".to_string(),
        "dev-12345.okta.com".to_string(),
    );

    match OktaProvider::new(okta_config) {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());

            if let Ok((auth_url, _)) = provider.authorization_url() {
                println!(
                    "   ✓ Auth URL: {}...",
                    &auth_url.as_str()[..80.min(auth_url.as_str().len())]
                );
            }
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // 5. Auth0
    println!("5. Auth0 Provider");
    println!("   Configuration:");
    let auth0_config = Auth0Config::new(
        env_or_placeholder("AUTH0_CLIENT_ID", "your-client-id"),
        env_or_placeholder("AUTH0_CLIENT_SECRET", "your-client-secret"),
        "http://localhost:3000/auth/auth0/callback".to_string(),
        env_or_placeholder("AUTH0_DOMAIN", "your-tenant.us.auth0.com"),
    )
    .with_audience("https://api.example.com".to_string());

    match Auth0Provider::new(auth0_config) {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());

            if let Ok((auth_url, _)) = provider.authorization_url() {
                println!(
                    "   ✓ Auth URL: {}...",
                    &auth_url.as_str()[..80.min(auth_url.as_str().len())]
                );
            }

            println!("   ℹ Optional audience parameter for API access");
        }
        Err(e) => println!("   ✗ Error: {}", e),
    }
    println!();

    // Summary
    println!("✅ OAuth2 Provider Summary:");
    println!("   ✓ Google - Consumer identity");
    println!("   ✓ Microsoft Entra - Enterprise Azure AD");
    println!("   ✓ AWS Cognito - AWS-native identity");
    println!("   ✓ Okta - Enterprise SSO");
    println!("   ✓ Auth0 - Universal authentication");
    println!();

    println!("🔄 OAuth2 Flow:");
    println!("   1. Generate authorization URL");
    println!("   2. Redirect user to provider");
    println!("   3. User authenticates and authorizes");
    println!("   4. Provider redirects back with code");
    println!("   5. Exchange code for access token");
    println!("   6. Fetch user info with access token");
    println!();

    println!("📖 For complete integration examples, see:");
    println!("   - docs/auth-guide.md");
    println!("   - docs/oauth2-guide.md (coming soon)");
    println!();

    println!("🔧 Setup Instructions:");
    println!();
    println!("Google:");
    println!("  1. Go to Google Cloud Console");
    println!("  2. Create OAuth 2.0 credentials");
    println!("  3. Add redirect URI");
    println!();
    println!("Microsoft Entra:");
    println!("  1. Go to Azure Portal");
    println!("  2. Register application in Azure AD");
    println!("  3. Configure redirect URI");
    println!();
    println!("AWS Cognito:");
    println!("  1. Create User Pool in AWS Console");
    println!("  2. Create App Client");
    println!("  3. Configure hosted UI");
    println!();
    println!("Okta:");
    println!("  1. Create application in Okta Admin");
    println!("  2. Configure redirect URI");
    println!("  3. Note domain and client credentials");
    println!();
    println!("Auth0:");
    println!("  1. Create application in Auth0 Dashboard");
    println!("  2. Configure callback URL");
    println!("  3. Set API audience if needed");
}
