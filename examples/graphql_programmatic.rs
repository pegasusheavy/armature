// Programmatic GraphQL schema example (NestJS-style)

use armature::prelude::*;
use armature_graphql::{
    EmptySubscription, ID, Object, ProgrammaticSchemaBuilder, Result, Schema, SimpleObject,
    async_graphql,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ========== Domain Models ==========

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
struct User {
    id: ID,
    #[graphql(desc = "User's full name")]
    name: String,
    #[graphql(desc = "User's email address")]
    email: String,
    #[graphql(desc = "User's role")]
    role: UserRole,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, async_graphql::Enum)]
enum UserRole {
    Admin,
    User,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::InputObject)]
struct CreateUserInput {
    name: String,
    email: String,
    role: Option<UserRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize, SimpleObject)]
struct Post {
    id: ID,
    title: String,
    content: String,
    author_id: ID,
}

#[derive(Debug, Clone, Serialize, Deserialize, async_graphql::InputObject)]
struct CreatePostInput {
    title: String,
    content: String,
    author_id: ID,
}

// ========== Services (Injectable) ==========

#[injectable]
#[derive(Default, Clone)]
struct UserService;

impl UserService {
    fn get_users(&self) -> Vec<User> {
        vec![
            User {
                id: ID::from("1"),
                name: "Alice".to_string(),
                email: "alice@example.com".to_string(),
                role: UserRole::Admin,
            },
            User {
                id: ID::from("2"),
                name: "Bob".to_string(),
                email: "bob@example.com".to_string(),
                role: UserRole::User,
            },
        ]
    }

    fn get_user_by_id(&self, id: &str) -> Option<User> {
        self.get_users().into_iter().find(|u| u.id.as_str() == id)
    }

    fn create_user(&self, input: CreateUserInput) -> User {
        User {
            id: ID::from("3"),
            name: input.name,
            email: input.email,
            role: input.role.unwrap_or(UserRole::User),
        }
    }
}

#[injectable]
#[derive(Default, Clone)]
struct PostService;

impl PostService {
    fn get_posts(&self) -> Vec<Post> {
        vec![
            Post {
                id: ID::from("1"),
                title: "First Post".to_string(),
                content: "This is the first post".to_string(),
                author_id: ID::from("1"),
            },
            Post {
                id: ID::from("2"),
                title: "Second Post".to_string(),
                content: "This is the second post".to_string(),
                author_id: ID::from("2"),
            },
        ]
    }

    fn create_post(&self, input: CreatePostInput) -> Post {
        Post {
            id: ID::from("3"),
            title: input.title,
            content: input.content,
            author_id: input.author_id,
        }
    }
}

// ========== GraphQL Resolvers (NestJS-style) ==========

/// Query Root (combining all query resolvers)
#[derive(Clone)]
struct QueryRoot {
    user_service: UserService,
    post_service: PostService,
}

#[Object]
impl QueryRoot {
    /// Get all users
    #[graphql(desc = "Retrieve all users")]
    async fn users(&self) -> Vec<User> {
        self.user_service.get_users()
    }

    /// Get a user by ID
    #[graphql(desc = "Retrieve a user by their ID")]
    async fn user(&self, #[graphql(desc = "User ID")] id: ID) -> Result<User> {
        self.user_service
            .get_user_by_id(id.as_str())
            .ok_or_else(|| "User not found".into())
    }

    /// Search users by name
    #[graphql(desc = "Search users by name")]
    async fn search_users(&self, query: String) -> Vec<User> {
        self.user_service
            .get_users()
            .into_iter()
            .filter(|u| u.name.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    /// Get users by role
    async fn users_by_role(&self, role: UserRole) -> Vec<User> {
        self.user_service
            .get_users()
            .into_iter()
            .filter(|u| u.role == role)
            .collect()
    }

    /// Get all posts
    async fn posts(&self) -> Vec<Post> {
        self.post_service.get_posts()
    }

    /// Get posts by author
    async fn posts_by_author(&self, author_id: ID) -> Vec<Post> {
        self.post_service
            .get_posts()
            .into_iter()
            .filter(|p| p.author_id == author_id)
            .collect()
    }
}

/// Mutation Root (combining all mutations)
#[derive(Clone)]
struct MutationRoot {
    user_service: UserService,
    post_service: PostService,
}

#[Object]
impl MutationRoot {
    /// Create a new user
    async fn create_user(&self, input: CreateUserInput) -> User {
        self.user_service.create_user(input)
    }

    /// Update user (simplified)
    async fn update_user(&self, id: ID, name: String) -> Result<User> {
        let mut user = self
            .user_service
            .get_user_by_id(id.as_str())
            .ok_or_else(|| "User not found")?;
        user.name = name;
        Ok(user)
    }

    /// Delete user
    async fn delete_user(&self, id: ID) -> bool {
        self.user_service.get_user_by_id(id.as_str()).is_some()
    }

    /// Create a new post
    async fn create_post(&self, input: CreatePostInput) -> Post {
        self.post_service.create_post(input)
    }
}

// ========== Application ==========

#[tokio::main]
async fn main() {
    println!("📊 Armature Programmatic GraphQL Example (NestJS-style)");
    println!("========================================================\n");

    let app = create_graphql_app();

    println!("GraphQL endpoint: http://localhost:3009/graphql");
    println!("GraphQL playground: http://localhost:3009/playground");
    println!();
    println!("Example queries:");
    println!();
    println!("1. Get all users:");
    println!("   query {{ users {{ id name email role }} }}");
    println!();
    println!("2. Search users:");
    println!("   query {{ searchUsers(query: \"Alice\") {{ id name }} }}");
    println!();
    println!("3. Get users by role:");
    println!("   query {{ usersByRole(role: ADMIN) {{ id name role }} }}");
    println!();
    println!("4. Get user with their posts:");
    println!("   query {{");
    println!("     user(id: \"1\") {{ id name }}");
    println!("     postsByAuthor(authorId: \"1\") {{ id title }}");
    println!("   }}");
    println!();
    println!("5. Create a user:");
    println!("   mutation {{");
    println!("     createUser(input: {{ name: \"Charlie\", email: \"charlie@example.com\" }}) {{");
    println!("       id name email");
    println!("     }}");
    println!("   }}");
    println!();

    if let Err(e) = app.listen(3009).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_graphql_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let user_service = UserService::default();
    let post_service = PostService::default();
    container.register(user_service.clone());
    container.register(post_service.clone());

    // Create resolvers with injected services
    let query_root = QueryRoot {
        user_service: user_service.clone(),
        post_service: post_service.clone(),
    };

    let mutation_root = MutationRoot {
        user_service: user_service.clone(),
        post_service: post_service.clone(),
    };

    // Build schema programmatically using the builder
    let schema = ProgrammaticSchemaBuilder::new()
        .query(query_root)
        .mutation(mutation_root)
        .subscription(EmptySubscription)
        .add_service(user_service)
        .add_service(post_service)
        .build();

    // GraphQL endpoint
    let schema_clone = schema.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/graphql".to_string(),
        handler: Arc::new(move |req| {
            let schema = schema_clone.clone();
            Box::pin(async move {
                #[derive(Deserialize)]
                struct GraphQLRequest {
                    query: String,
                    #[serde(default)]
                    variables: Option<serde_json::Value>,
                    #[serde(default)]
                    operation_name: Option<String>,
                }

                let gql_req: GraphQLRequest = req.json()?;
                let mut request = async_graphql::Request::new(gql_req.query);

                if let Some(vars) = gql_req.variables {
                    request = request.variables(async_graphql::Variables::from_json(vars));
                }
                if let Some(op_name) = gql_req.operation_name {
                    request = request.operation_name(op_name);
                }

                let response = schema.execute(request).await;
                let json = serde_json::to_value(&response)
                    .map_err(|e| Error::Serialization(e.to_string()))?;

                HttpResponse::ok().with_json(&json)
            })
        }),
    });

    // GraphQL Playground
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/playground".to_string(),
        handler: Arc::new(move |_req| {
            Box::pin(async move {
                let html = armature_graphql::graphiql_html("/graphql");
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "text/html".to_string())
                    .with_body(html.into_bytes()))
            })
        }),
    });

    // Schema SDL endpoint
    let schema_clone = schema.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/graphql/schema".to_string(),
        handler: Arc::new(move |_req| {
            let schema = schema_clone.clone();
            Box::pin(async move {
                let sdl = schema.sdl();
                Ok(HttpResponse::ok()
                    .with_header("Content-Type".to_string(), "text/plain".to_string())
                    .with_body(sdl.into_bytes()))
            })
        }),
    });

    Application {
        container,
        router: Arc::new(router),
    }
}
