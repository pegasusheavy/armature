/// Example demonstrating GraphQL with documentation and configurable playgrounds
use armature::prelude::*;
use armature_graphql::{
    async_graphql, generate_schema_docs_html, graphiql_html, graphql_playground_html,
    EmptyMutation, EmptySubscription, GraphQLConfig, Object, Schema, SimpleObject,
};

// Define your GraphQL types
#[derive(SimpleObject, Clone)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[derive(SimpleObject, Clone)]
struct Post {
    id: i32,
    title: String,
    content: String,
    author_id: i32,
}

// Define the Query root
struct Query;

#[Object]
impl Query {
    /// Get a user by ID
    async fn user(&self, id: i32) -> Option<User> {
        // In a real app, this would query a database
        Some(User {
            id,
            name: format!("User {}", id),
            email: format!("user{}@example.com", id),
        })
    }

    /// Get all users
    async fn users(&self) -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
            },
        ]
    }

    /// Get a post by ID
    async fn post(&self, id: i32) -> Option<Post> {
        Some(Post {
            id,
            title: format!("Post {}", id),
            content: format!("Content for post {}", id),
            author_id: 1,
        })
    }

    /// Get all posts
    async fn posts(&self) -> Vec<Post> {
        vec![
            Post {
                id: 1,
                title: "First Post".to_string(),
                content: "This is the first post".to_string(),
                author_id: 1,
            },
            Post {
                id: 2,
                title: "Second Post".to_string(),
                content: "This is the second post".to_string(),
                author_id: 2,
            },
        ]
    }
}

// Injectable GraphQL service
#[injectable]
struct GraphQLService {
    schema: Schema<Query, EmptyMutation, EmptySubscription>,
    config: GraphQLConfig,
}

impl GraphQLService {
    fn new() -> Self {
        let schema = Schema::build(Query, EmptyMutation, EmptySubscription).finish();

        // Configure GraphQL with all features enabled for development
        let config = GraphQLConfig::development("/api/graphql")
            .with_playground(true)
            .with_graphiql(true)
            .with_schema_docs(true);

        Self { schema, config }
    }

    async fn execute_query(&self, query: &str) -> String {
        let request = async_graphql::Request::new(query);
        let response = self.schema.execute(request).await;
        serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
    }
}

// GraphQL controller
#[controller("/api/graphql")]
struct GraphQLController {
    service: GraphQLService,
}

impl GraphQLController {
    fn new(service: GraphQLService) -> Self {
        Self { service }
    }

    /// Main GraphQL endpoint
    #[post("/")]
    async fn graphql_endpoint(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        // Parse the GraphQL query from request body
        let body = req.body();
        let query_data: serde_json::Value =
            serde_json::from_slice(&body).map_err(|e| Error::BadRequest(e.to_string()))?;

        let query = query_data["query"]
            .as_str()
            .ok_or_else(|| Error::BadRequest("Missing query field".to_string()))?;

        // Execute the query
        let result = self.service.execute_query(query).await;

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "application/json".to_string())
            .with_body(result.into_bytes()))
    }

    /// GraphQL Playground (only if enabled)
    #[get("/playground")]
    async fn playground(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        if !self.service.config.enable_playground {
            return Err(Error::NotFound);
        }

        let html = graphql_playground_html(&self.service.config.endpoint);

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/html".to_string())
            .with_body(html.into_bytes()))
    }

    /// GraphiQL (only if enabled)
    #[get("/graphiql")]
    async fn graphiql(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        if !self.service.config.enable_graphiql {
            return Err(Error::NotFound);
        }

        let html = graphiql_html(&self.service.config.endpoint);

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/html".to_string())
            .with_body(html.into_bytes()))
    }

    /// Schema documentation endpoint (only if enabled)
    #[get("/schema")]
    async fn schema_docs(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        if !self.service.config.enable_schema_docs {
            return Err(Error::NotFound);
        }

        let html = generate_schema_docs_html(
            &self.service.schema,
            &self.service.config.endpoint,
            "Blog API",
        );

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/html".to_string())
            .with_body(html.into_bytes()))
    }

    /// Schema SDL download
    #[get("/schema.graphql")]
    async fn schema_sdl(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let sdl = self.service.schema.sdl();

        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/plain".to_string())
            .with_header(
                "Content-Disposition".to_string(),
                "attachment; filename=\"schema.graphql\"".to_string(),
            )
            .with_body(sdl.into_bytes()))
    }
}

// Module configuration
#[module]
struct AppModule {
    providers: [GraphQLService],
    controllers: [GraphQLController],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting GraphQL server with documentation...\n");
    println!("📍 Endpoints:");
    println!("   - GraphQL API:        http://localhost:4000/api/graphql");
    println!("   - Playground:         http://localhost:4000/api/graphql/playground");
    println!("   - GraphiQL:           http://localhost:4000/api/graphql/graphiql");
    println!("   - Schema Docs:        http://localhost:4000/api/graphql/schema");
    println!("   - Schema SDL:         http://localhost:4000/api/graphql/schema.graphql");
    println!("\n💡 Try these queries in the playground:");
    println!("   query {{ users {{ id name email }} }}");
    println!("   query {{ posts {{ id title content authorId }} }}");
    println!("   query {{ user(id: 1) {{ name email }} }}\n");

    let app = Application::create::<AppModule>();
    app.listen(4000).await?;

    Ok(())
}

