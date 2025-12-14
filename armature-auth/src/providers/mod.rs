// OAuth2 provider implementations

pub mod auth0;
pub mod aws_cognito;
pub mod discord;
pub mod github;
pub mod gitlab;
pub mod google;
pub mod linkedin;
pub mod microsoft;
pub mod okta;

pub use auth0::{Auth0Config, Auth0Provider};
pub use aws_cognito::{AwsCognitoConfig, AwsCognitoProvider};
pub use discord::{DiscordProvider, DiscordUser};
pub use github::{GitHubProvider, GitHubUser};
pub use gitlab::{GitLabProvider, GitLabUser};
pub use google::{GoogleConfig, GoogleProvider};
pub use linkedin::{LinkedInProvider, LinkedInUser};
pub use microsoft::{MicrosoftEntraConfig, MicrosoftEntraProvider};
pub use okta::{OktaConfig, OktaProvider};
