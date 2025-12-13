//! GraphQL schema definitions

mod mutation;
mod query;
mod types;

pub use mutation::MutationRoot;
pub use query::QueryRoot;
pub use types::*;
