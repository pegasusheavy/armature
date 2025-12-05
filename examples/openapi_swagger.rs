use armature::prelude::*;
use armature::armature_openapi::*;

// Example user service
#[derive(Clone)]
#[injectable]
struct UserService;

// Example controller
#[controller("/api/users")]
struct UserController {
    _user_service: UserService,
}

impl UserController {
    #[get("/")]
    async fn list_users(&self) -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "users": ["Alice", "Bob", "Charlie"]
        })))
    }

    #[get("/:id")]
    async fn get_user(&self, #[Param("id")] id: u64) -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "id": id,
            "name": "Alice",
            "email": "alice@example.com"
        })))
    }

    #[post("/")]
    async fn create_user(
        &self,
        #[Body] body: Json<serde_json::Value>,
    ) -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "id": 1,
            "name": body.get("name"),
            "email": body.get("email")
        })))
    }
}

// API Documentation controller
#[controller("/api-docs")]
struct ApiDocsController {
    config: SwaggerConfig,
}

impl ApiDocsController {
    #[get("/")]
    async fn swagger_ui(&self) -> Result<HttpResponse, Error> {
        swagger_ui_response(&self.config)
    }

    #[get("/openapi.json")]
    async fn openapi_json(&self) -> Result<HttpResponse, Error> {
        spec_json_response(&self.config)
    }

    #[get("/openapi.yaml")]
    async fn openapi_yaml(&self) -> Result<HttpResponse, Error> {
        spec_yaml_response(&self.config)
    }
}

// Application module
#[module]
struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📚 Building OpenAPI specification...");

    // Build OpenAPI specification programmatically
    let spec = OpenApiBuilder::new("User API", "1.0.0")
        .description("A simple user management API")
        .server("http://localhost:3000", Some("Development server".to_string()))
        .tag("users", Some("User management endpoints".to_string()))
        // Add Bearer authentication
        .add_bearer_auth("bearer")
        // Add user schema
        .schema(
            "User",
            object_schema(
                {
                    let mut props = std::collections::HashMap::new();
                    props.insert("id".to_string(), integer_schema());
                    props.insert("name".to_string(), string_schema());
                    props.insert("email".to_string(), string_schema());
                    props
                },
                vec!["id".to_string(), "name".to_string(), "email".to_string()],
            ),
        )
        // Add GET /api/users endpoint
        .path(
            "/api/users",
            PathItemBuilder::new()
                .get(
                    OperationBuilder::new()
                        .summary("List all users")
                        .description("Returns a list of all users")
                        .tag("users")
                        .operation_id("listUsers")
                        .response(
                            "200",
                            Response {
                                description: "Successful response".to_string(),
                                content: Some({
                                    let mut content = std::collections::HashMap::new();
                                    content.insert(
                                        "application/json".to_string(),
                                        MediaType {
                                            schema: Some(object_schema(
                                                {
                                                    let mut props = std::collections::HashMap::new();
                                                    props.insert(
                                                        "users".to_string(),
                                                        array_schema(ref_schema("User")),
                                                    );
                                                    props
                                                },
                                                vec!["users".to_string()],
                                            )),
                                        },
                                    );
                                    content
                                }),
                            },
                        )
                        .build(),
                )
                .post(
                    OperationBuilder::new()
                        .summary("Create a user")
                        .description("Creates a new user")
                        .tag("users")
                        .operation_id("createUser")
                        .request_body(RequestBody {
                            description: Some("User to create".to_string()),
                            content: {
                                let mut content = std::collections::HashMap::new();
                                content.insert(
                                    "application/json".to_string(),
                                    MediaType {
                                        schema: Some(ref_schema("User")),
                                    },
                                );
                                content
                            },
                            required: Some(true),
                        })
                        .response(
                            "201",
                            Response {
                                description: "User created".to_string(),
                                content: Some({
                                    let mut content = std::collections::HashMap::new();
                                    content.insert(
                                        "application/json".to_string(),
                                        MediaType {
                                            schema: Some(ref_schema("User")),
                                        },
                                    );
                                    content
                                }),
                            },
                        )
                        .build(),
                )
                .build(),
        )
        // Add GET /api/users/:id endpoint
        .path(
            "/api/users/{id}",
            PathItemBuilder::new()
                .get(
                    OperationBuilder::new()
                        .summary("Get a user by ID")
                        .description("Returns a single user")
                        .tag("users")
                        .operation_id("getUserById")
                        .parameter(Parameter {
                            name: "id".to_string(),
                            location: ParameterLocation::Path,
                            description: Some("User ID".to_string()),
                            required: Some(true),
                            schema: Some(integer_schema()),
                        })
                        .response(
                            "200",
                            Response {
                                description: "Successful response".to_string(),
                                content: Some({
                                    let mut content = std::collections::HashMap::new();
                                    content.insert(
                                        "application/json".to_string(),
                                        MediaType {
                                            schema: Some(ref_schema("User")),
                                        },
                                    );
                                    content
                                }),
                            },
                        )
                        .response(
                            "404",
                            Response {
                                description: "User not found".to_string(),
                                content: None,
                            },
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    println!("✅ OpenAPI specification built");

    // Create Swagger configuration
    let swagger_config = SwaggerConfig::new("/api-docs", spec)
        .with_title("User API Documentation");

    // Register Swagger UI controller with config
    let (mut container, mut router) = Application::create::<AppModule>().await?;

    // Manually register ApiDocsController with config
    container.register(ApiDocsController {
        config: swagger_config,
    });

    // Register routes for API docs
    // Note: In a real app, this would be done via the controller macro
    // but we're manually adding it here for demonstration

    let app = Application::new(container, router);

    println!("🚀 Server running on http://localhost:3000");
    println!("📖 API Documentation:");
    println!("   Swagger UI:    http://localhost:3000/api-docs");
    println!("   OpenAPI JSON:  http://localhost:3000/api-docs/openapi.json");
    println!("   OpenAPI YAML:  http://localhost:3000/api-docs/openapi.yaml");
    println!("\n📝 Try the API:");
    println!("   GET  http://localhost:3000/api/users");
    println!("   GET  http://localhost:3000/api/users/1");
    println!("   POST http://localhost:3000/api/users");

    app.listen("0.0.0.0:3000").await?;

    Ok(())
}

