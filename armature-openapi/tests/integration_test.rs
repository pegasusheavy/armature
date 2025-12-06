//! Integration tests for armature-openapi

use armature_openapi::*;

#[test]
fn test_openapi_builder_creation() {
    let builder = OpenApiBuilder::new("My API", "1.0.0");
    let spec = builder.build();

    assert_eq!(spec.openapi, "3.0.0");
    assert_eq!(spec.info.title, "My API");
    assert_eq!(spec.info.version, "1.0.0");
}

#[test]
fn test_openapi_builder_with_description() {
    let builder = OpenApiBuilder::new("My API", "1.0.0")
        .description("API Description");
    let spec = builder.build();

    assert_eq!(spec.info.description, Some("API Description".to_string()));
}

#[test]
fn test_openapi_builder_with_server() {
    let builder = OpenApiBuilder::new("My API", "1.0.0")
        .server("https://api.example.com", Some("Production".to_string()));
    let spec = builder.build();

    assert_eq!(spec.servers.len(), 1);
    assert_eq!(spec.servers[0].url, "https://api.example.com");
}

#[test]
fn test_openapi_builder_with_path() {
    let builder = OpenApiBuilder::new("My API", "1.0.0")
        .path("/users", PathItem::default());
    let spec = builder.build();

    assert!(spec.paths.contains_key("/users"));
}

#[test]
fn test_openapi_info_creation() {
    let info = Info {
        title: "Test API".to_string(),
        version: "1.0.0".to_string(),
        description: Some("Description".to_string()),
        terms_of_service: None,
        contact: None,
        license: None,
    };

    assert_eq!(info.title, "Test API");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn test_server_creation() {
    let server = Server {
        url: "https://api.example.com".to_string(),
        description: Some("Production".to_string()),
        variables: None,
    };

    assert_eq!(server.url, "https://api.example.com");
}

#[test]
fn test_path_item_creation() {
    let path = PathItem {
        get: Some(Operation::default()),
        post: None,
        put: None,
        delete: None,
        options: None,
        head: None,
        patch: None,
        parameters: vec![],
    };

    assert!(path.get.is_some());
    assert!(path.post.is_none());
}

#[test]
fn test_operation_creation() {
    let op = Operation {
        tags: vec!["users".to_string()],
        summary: Some("Get user".to_string()),
        description: Some("Get user by ID".to_string()),
        operation_id: Some("getUser".to_string()),
        parameters: vec![],
        request_body: None,
        responses: std::collections::HashMap::new(),
        security: vec![],
    };

    assert_eq!(op.tags.len(), 1);
    assert_eq!(op.summary, Some("Get user".to_string()));
}

#[test]
fn test_schema_object_string() {
    let schema = Schema::Object(SchemaObject {
        schema_type: Some("string".to_string()),
        format: None,
        description: Some("A string field".to_string()),
        properties: std::collections::HashMap::new(),
        required: vec![],
        items: None,
        enum_values: None,
        example: None,
    });

    if let Schema::Object(obj) = schema {
        assert_eq!(obj.schema_type, Some("string".to_string()));
    }
}

#[test]
fn test_schema_ref() {
    let schema = Schema::Ref("#/components/schemas/User".to_string());

    if let Schema::Ref(ref_str) = schema {
        assert_eq!(ref_str, "#/components/schemas/User");
    }
}

#[test]
fn test_swagger_config_default() {
    let config = SwaggerConfig::default();

    assert_eq!(config.url, "/api-docs");
    assert_eq!(config.spec_url, "/api-docs/openapi.json");
    assert_eq!(config.title, "API Documentation");
}

#[test]
fn test_swagger_config_builder() {
    let config = SwaggerConfig::new("/docs")
        .spec_url("/docs/spec.json")
        .title("My API Docs");

    assert_eq!(config.url, "/docs");
    assert_eq!(config.spec_url, "/docs/spec.json");
    assert_eq!(config.title, "My API Docs");
}

#[test]
fn test_swagger_ui_response() {
    let config = SwaggerConfig::default();
    let html = swagger_ui_response(&config);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("swagger-ui"));
    assert!(html.contains(&config.spec_url));
}

#[test]
fn test_openapi_spec_serialization() {
    let builder = OpenApiBuilder::new("My API", "1.0.0");
    let spec = builder.build();

    let json = serde_json::to_string(&spec);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("My API"));
    assert!(json_str.contains("1.0.0"));
}


