// OAuth2 provider implementations

pub mod auth0;
pub mod aws_cognito;
pub mod google;
pub mod microsoft;
pub mod okta;

pub use auth0::{Auth0Config, Auth0Provider};
pub use aws_cognito::{AwsCognitoConfig, AwsCognitoProvider};
pub use google::{GoogleConfig, GoogleProvider};
pub use microsoft::{MicrosoftEntraConfig, MicrosoftEntraProvider};
pub use okta::{OktaConfig, OktaProvider};
