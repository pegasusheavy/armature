#![allow(dead_code)]
// Simple SAML 2.0 example

use armature_auth::saml::{IdpMetadata, SamlConfig, SamlProvider, SamlServiceProvider};

#[tokio::main]
async fn main() {
    println!("🔐 Armature SAML 2.0 Example");
    println!("============================\n");

    // 1. Basic SAML Configuration
    println!("1. Configuring SAML Service Provider");

    let config = SamlConfig::new(
        "https://myapp.example.com/saml/metadata".to_string(),
        "https://myapp.example.com/saml/acs".to_string(),
        IdpMetadata::Xml(SAMPLE_IDP_METADATA.to_string()),
    )
    .with_sls_url("https://myapp.example.com/saml/sls".to_string())
    .allow_unsigned(false)
    .with_required_attributes(vec!["email".to_string(), "displayName".to_string()]);

    println!("   Entity ID: {}", config.entity_id);
    println!("   ACS URL: {}", config.acs_url);
    println!("   SLS URL: {}", config.sls_url.as_ref().unwrap());
    println!("   Unsigned allowed: {}", config.allow_unsigned_assertions);
    println!();

    // 2. Create SAML Provider
    println!("2. Creating SAML Provider");
    let provider = SamlServiceProvider::new("okta".to_string(), config);

    match provider {
        Ok(provider) => {
            println!("   ✓ Provider: {}", provider.name());
            println!("   ✓ SAML SP created successfully\n");

            // 3. Generate Authentication Request
            println!("3. Generating SAML Authentication Request");
            match provider.create_auth_request() {
                Ok(auth_req) => {
                    println!("   ✓ SAMLRequest generated");
                    println!("   ✓ Redirect URL: {}", auth_req.redirect_url);
                    println!(
                        "   ✓ Relay State: {}",
                        auth_req.relay_state.unwrap_or_default()
                    );
                    println!(
                        "   ✓ SAML Request (first 100 chars): {}...",
                        &auth_req.saml_request[..100.min(auth_req.saml_request.len())]
                    );
                    println!();
                }
                Err(e) => println!("   ✗ Error: {}\n", e),
            }

            // 4. Get SP Metadata
            println!("4. Generating SP Metadata");
            match provider.get_metadata() {
                Ok(metadata) => {
                    println!("   ✓ Metadata XML generated");
                    let lines: Vec<&str> = metadata.lines().collect();
                    println!("   ✓ Metadata (first 5 lines):");
                    for (i, line) in lines.iter().take(5).enumerate() {
                        println!("      {}: {}", i + 1, line);
                    }
                    println!();
                }
                Err(e) => println!("   ✗ Error: {}\n", e),
            }

            // 5. SAML Flow Overview
            println!("5. SAML 2.0 Flow:");
            println!("   1. User clicks 'Login with SSO'");
            println!("   2. SP generates AuthnRequest");
            println!("   3. User redirected to IdP");
            println!("   4. User authenticates at IdP");
            println!("   5. IdP posts SAML Response to SP ACS");
            println!("   6. SP validates assertion");
            println!("   7. User logged in");
            println!();
        }
        Err(e) => println!("   ✗ Error creating provider: {}\n", e),
    }

    // 6. Summary
    println!("✅ SAML 2.0 Features:");
    println!("   ✓ SP configuration");
    println!("   ✓ AuthnRequest generation");
    println!("   ✓ Assertion validation");
    println!("   ✓ Metadata generation");
    println!("   ✓ Enterprise SSO support");
    println!();

    println!("📖 Common SAML Providers:");
    println!("   - Okta");
    println!("   - Microsoft Entra (Azure AD)");
    println!("   - Auth0");
    println!("   - OneLogin");
    println!("   - Ping Identity");
    println!();

    println!("🔧 Setup Steps:");
    println!("   1. Register SP in IdP");
    println!("   2. Download IdP metadata");
    println!("   3. Configure SP with IdP metadata");
    println!("   4. Upload SP metadata to IdP");
    println!("   5. Test authentication");
    println!();

    println!("📄 For complete integration, see:");
    println!("   - docs/AUTH_GUIDE.md");
    println!("   - docs/SAML_GUIDE.md (coming soon)");
}

// Sample IdP metadata (simplified for demo)
const SAMPLE_IDP_METADATA: &str = r#"<?xml version="1.0"?>
<EntityDescriptor xmlns="urn:oasis:names:tc:SAML:2.0:metadata" entityID="https://idp.example.com">
  <IDPSSODescriptor protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect"
                         Location="https://idp.example.com/sso"/>
    <SingleSignOnService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST"
                         Location="https://idp.example.com/sso"/>
  </IDPSSODescriptor>
</EntityDescriptor>"#;
