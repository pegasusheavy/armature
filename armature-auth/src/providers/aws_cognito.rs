// AWS Cognito OAuth2 provider

use crate::Result;
use crate::oauth2::{
    GenericOAuth2Provider, OAuth2Config, OAuth2Provider, OAuth2Token, OAuth2UserInfo,
};
use async_trait::async_trait;
use oauth2::CsrfToken;
use url::Url;

/// AWS Cognito provider configuration
#[derive(Debug, Clone)]
pub struct AwsCognitoConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_url: String,
    pub user_pool_domain: String, // e.g., "my-app.auth.us-east-1.amazoncognito.com"
    pub region: String,           // e.g., "us-east-1"
    pub scopes: Vec<String>,
}

impl AwsCognitoConfig {
    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_url: String,
        user_pool_domain: String,
        region: String,
    ) -> Self {
        Self {
            client_id,
            client_secret,
            redirect_url,
            user_pool_domain,
            region,
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
}

/// AWS Cognito provider
pub struct AwsCognitoProvider {
    inner: GenericOAuth2Provider,
}

impl AwsCognitoProvider {
    pub fn new(config: AwsCognitoConfig) -> Result<Self> {
        let auth_url = format!("https://{}/oauth2/authorize", config.user_pool_domain);
        let token_url = format!("https://{}/oauth2/token", config.user_pool_domain);
        let user_info_url = format!("https://{}/oauth2/userInfo", config.user_pool_domain);

        let oauth2_config = OAuth2Config::new(
            config.client_id,
            config.client_secret,
            auth_url,
            token_url,
            config.redirect_url,
        )
        .with_scopes(config.scopes)
        .with_user_info_url(user_info_url);

        let inner = GenericOAuth2Provider::new("aws-cognito".to_string(), oauth2_config)?;

        Ok(Self { inner })
    }
}

#[async_trait]
impl OAuth2Provider for AwsCognitoProvider {
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
        self.inner.get_user_info(token).await
    }

    async fn refresh_token(&self, refresh_token: String) -> Result<OAuth2Token> {
        self.inner.refresh_token(refresh_token).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cognito_config() {
        let config = AwsCognitoConfig::new(
            "client_id".to_string(),
            "client_secret".to_string(),
            "http://localhost:3000/callback".to_string(),
            "my-app.auth.us-east-1.amazoncognito.com".to_string(),
            "us-east-1".to_string(),
        );

        assert_eq!(config.region, "us-east-1");
        assert_eq!(config.scopes.len(), 3);
    }
}
