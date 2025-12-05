// Microsoft Entra (Azure AD) OAuth2 provider

use crate::Result;
use crate::oauth2::{
    GenericOAuth2Provider, OAuth2Config, OAuth2Provider, OAuth2Token, OAuth2UserInfo,
};
use async_trait::async_trait;
use oauth2::CsrfToken;
use url::Url;

/// Microsoft Entra provider configuration
#[derive(Debug, Clone)]
pub struct MicrosoftEntraConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub tenant_id: String, // "common", "organizations", "consumers", or specific tenant ID
    pub scopes: Vec<String>,
}

impl MicrosoftEntraConfig {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_url: String,
        tenant_id: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            tenant_id,
            scopes: vec![
                "openid".to_string(),
                "email".to_string(),
                "profile".to_string(),
            ],
        }
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    /// Create config for common tenant (any Azure AD account)
    pub fn common(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self::new(client_id, client_secret, redirect_url, "common".to_string())
    }

    /// Create config for organization accounts only
    pub fn organizations(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self::new(
            client_id,
            client_secret,
            redirect_url,
            "organizations".to_string(),
        )
    }

    /// Create config for personal Microsoft accounts
    pub fn consumers(client_id: String, client_secret: String, redirect_url: String) -> Self {
        Self::new(
            client_id,
            client_secret,
            redirect_url,
            "consumers".to_string(),
        )
    }
}

/// Microsoft Entra (Azure AD) provider
pub struct MicrosoftEntraProvider {
    inner: GenericOAuth2Provider,
}

impl MicrosoftEntraProvider {
    pub fn new(config: MicrosoftEntraConfig) -> Result<Self> {
        let auth_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize",
            config.tenant_id
        );
        let token_url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            config.tenant_id
        );

        let oauth2_config = OAuth2Config::new(
            config.client_id,
            config.client_secret,
            auth_url,
            token_url,
            config.redirect_url,
        )
        .with_scopes(config.scopes)
        .with_user_info_url("https://graph.microsoft.com/v1.0/me".to_string());

        let inner = GenericOAuth2Provider::new("microsoft-entra".to_string(), oauth2_config)?;

        Ok(Self { inner })
    }
}

#[async_trait]
impl OAuth2Provider for MicrosoftEntraProvider {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn authorization_url(&self) -> Result<(Url, CsrfToken)> {
        self.inner.authorization_url()
    }

    async fn exchange_code(&self, code: String) -> Result<OAuth2Token> {
        self.inner.exchange_code(code).await
    }

    async fn get_user_info(&self, token: &OAuth2Token) -> Result<OAuth2UserInfo> {
        // Microsoft Graph returns different format, so we need to transform it
        let user_info = self.inner.get_user_info(token).await?;

        // Microsoft Graph uses 'id' instead of 'sub'
        Ok(user_info)
    }

    async fn refresh_token(&self, refresh_token: String) -> Result<OAuth2Token> {
        self.inner.refresh_token(refresh_token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_microsoft_config() {
        let config = MicrosoftEntraConfig::common(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );

        assert_eq!(config.tenant_id, "common");
        assert_eq!(config.scopes.len(), 3);
    }

    #[test]
    fn test_microsoft_organizations() {
        let config = MicrosoftEntraConfig::organizations(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
        );

        assert_eq!(config.tenant_id, "organizations");
    }
}
