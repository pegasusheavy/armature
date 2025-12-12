//! Integration tests for armature-graphql

use armature_graphql::*;

#[test]
fn test_graphql_config_default() {
    let config = GraphQLConfig::default();

    assert_eq!(config.endpoint, "/graphql");
    assert!(config.enable_playground);
    assert!(config.enable_introspection);
}

#[test]
fn test_graphql_config_production() {
    let config = GraphQLConfig::production();

    assert!(!config.enable_playground);
    assert!(!config.enable_graphiql);
    assert!(!config.enable_introspection);
}

#[test]
fn test_graphql_config_development() {
    let config = GraphQLConfig::development();

    assert!(config.enable_playground);
    assert!(config.enable_graphiql);
    assert!(config.enable_introspection);
}

#[test]
fn test_graphql_config_builder() {
    let config = GraphQLConfig::new("/api/graphql")
        .with_playground(true)
        .with_graphiql(false)
        .with_schema_docs(true)
        .with_max_depth(10)
        .with_max_complexity(100)
        .with_validation(true)
        .with_tracing(false);

    assert_eq!(config.endpoint, "/api/graphql");
    assert!(config.enable_playground);
    assert!(!config.enable_graphiql);
    assert!(config.enable_schema_docs);
    assert_eq!(config.max_depth, 10);
    assert_eq!(config.max_complexity, 100);
    assert!(config.enable_validation);
    assert!(!config.enable_tracing);
}

#[test]
fn test_graphql_playground_html() {
    let html = graphql_playground_html("/graphql", "Test API");

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("GraphQL Playground"));
    assert!(html.contains("/graphql"));
}

#[test]
fn test_graphiql_html() {
    let html = graphiql_html("/graphql", "Test API");

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("GraphiQL"));
    assert!(html.contains("/graphql"));
}
