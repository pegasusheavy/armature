use armature_core::*;
use armature_handlebars::{HandlebarsConfig, HandlebarsService};
use armature_macro::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// --- Data Models ---

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: String,
    active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Post {
    id: u64,
    title: String,
    content: String,
    author: String,
    views: u64,
}

// --- Services ---

#[derive(Clone)]
struct UserService;

impl Provider for UserService {}

impl UserService {
    fn new() -> Self {
        Self
    }

    fn get_users(&self) -> Vec<User> {
        vec![
            User {
                id: 1,
                name: "Alice Smith".to_string(),
                email: "alice@example.com".to_string(),
                role: "admin".to_string(),
                active: true,
            },
            User {
                id: 2,
                name: "Bob Johnson".to_string(),
                email: "bob@example.com".to_string(),
                role: "user".to_string(),
                active: true,
            },
            User {
                id: 3,
                name: "Charlie Brown".to_string(),
                email: "charlie@example.com".to_string(),
                role: "user".to_string(),
                active: false,
            },
        ]
    }

    fn get_user(&self, id: u64) -> Option<User> {
        self.get_users().into_iter().find(|u| u.id == id)
    }
}

#[derive(Clone)]
struct PostService;

impl Provider for PostService {}

impl PostService {
    fn new() -> Self {
        Self
    }

    fn get_posts(&self) -> Vec<Post> {
        vec![
            Post {
                id: 1,
                title: "Getting Started with Armature".to_string(),
                content: "Armature is a modern Rust web framework...".to_string(),
                author: "Alice Smith".to_string(),
                views: 1250,
            },
            Post {
                id: 2,
                title: "Handlebars Templates".to_string(),
                content: "Learn how to use Handlebars for server-side rendering...".to_string(),
                author: "Bob Johnson".to_string(),
                views: 850,
            },
        ]
    }

    fn get_post(&self, id: u64) -> Option<Post> {
        self.get_posts().into_iter().find(|p| p.id == id)
    }
}

// --- Controllers ---

#[controller("/")]
#[derive(Clone)]
struct HomeController {
    handlebars: HandlebarsService,
}

impl HomeController {
    async fn index(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let data = serde_json::json!({
            "title": "Armature Handlebars Example",
            "description": "Server-side rendering with Handlebars templates",
        });

        self.handlebars
            .render_response("index", &data)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }
}

#[controller("/users")]
#[derive(Clone)]
struct UserController {
    handlebars: HandlebarsService,
    user_service: UserService,
}

impl UserController {
    async fn list_users(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let users = self.user_service.get_users();

        let data = serde_json::json!({
            "title": "Users",
            "users": users,
        });

        self.handlebars
            .render_response("users/list", &data)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }

    async fn get_user(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id_str = req.path_params.get("id").ok_or_else(|| {
            Error::BadRequest("Missing user id".to_string())
        })?;

        let id: u64 = id_str.parse().map_err(|_| {
            Error::BadRequest("Invalid user id".to_string())
        })?;

        let user = self.user_service.get_user(id).ok_or_else(|| {
            Error::NotFound("User not found".to_string())
        })?;

        let data = serde_json::json!({
            "title": format!("User: {}", user.name),
            "user": user,
        });

        self.handlebars
            .render_response("users/detail", &data)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }
}

#[controller("/posts")]
#[derive(Clone)]
struct PostController {
    handlebars: HandlebarsService,
    post_service: PostService,
}

impl PostController {
    async fn list_posts(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let posts = self.post_service.get_posts();

        let data = serde_json::json!({
            "title": "Blog Posts",
            "posts": posts,
        });

        self.handlebars
            .render_response("posts/list", &data)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }

    async fn get_post(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let id_str = req.path_params.get("id").ok_or_else(|| {
            Error::BadRequest("Missing post id".to_string())
        })?;

        let id: u64 = id_str.parse().map_err(|_| {
            Error::BadRequest("Invalid post id".to_string())
        })?;

        let post = self.post_service.get_post(id).ok_or_else(|| {
            Error::NotFound("Post not found".to_string())
        })?;

        let data = serde_json::json!({
            "title": post.title.clone(),
            "post": post,
        });

        self.handlebars
            .render_response("posts/detail", &data)
            .await
            .map_err(|e| Error::Internal(e.to_string()))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📝 Armature Handlebars Templates Example");
    println!("==========================================\n");

    // Setup demo templates
    setup_demo_templates().await?;

    // Configure Handlebars
    let handlebars_config = HandlebarsConfig::new("demo/templates")
        .with_extension(".hbs")
        .with_dev_mode(true); // Enable hot-reload in development

    let handlebars_service = HandlebarsService::new(handlebars_config)?;

    // Create services
    let user_service = UserService::new();
    let post_service = PostService::new();

    // Create router
    let container = Container::new();
    let mut router = Router::new();

    // Home route
    let home_ctrl = HomeController {
        handlebars: handlebars_service.clone(),
    };
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = home_ctrl.clone();
            Box::pin(async move { ctrl.index(req).await })
        }),
    });

    // User routes
    let users_list_ctrl = UserController {
        handlebars: handlebars_service.clone(),
        user_service: user_service.clone(),
    };
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/users".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = users_list_ctrl.clone();
            Box::pin(async move { ctrl.list_users(req).await })
        }),
    });

    let users_detail_ctrl = UserController {
        handlebars: handlebars_service.clone(),
        user_service: user_service.clone(),
    };
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/users/:id".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = users_detail_ctrl.clone();
            Box::pin(async move { ctrl.get_user(req).await })
        }),
    });

    // Post routes
    let posts_list_ctrl = PostController {
        handlebars: handlebars_service.clone(),
        post_service: post_service.clone(),
    };
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/posts".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = posts_list_ctrl.clone();
            Box::pin(async move { ctrl.list_posts(req).await })
        }),
    });

    let posts_detail_ctrl = PostController {
        handlebars: handlebars_service.clone(),
        post_service: post_service.clone(),
    };
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/posts/:id".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = posts_detail_ctrl.clone();
            Box::pin(async move { ctrl.get_post(req).await })
        }),
    });

    let app = Application {
        router: Arc::new(router),
        container,
    };

    println!("🚀 Server starting on http://localhost:3000");
    println!("\nRoutes:");
    println!("  • http://localhost:3000/           → Home page");
    println!("  • http://localhost:3000/users      → User list");
    println!("  • http://localhost:3000/users/1    → User detail");
    println!("  • http://localhost:3000/posts      → Post list");
    println!("  • http://localhost:3000/posts/1    → Post detail\n");

    app.listen(3000).await?;

    Ok(())
}

async fn setup_demo_templates() -> std::io::Result<()> {
    use tokio::fs;

    // Create directories
    fs::create_dir_all("demo/templates").await?;
    fs::create_dir_all("demo/templates/users").await?;
    fs::create_dir_all("demo/templates/posts").await?;
    fs::create_dir_all("demo/templates/partials").await?;

    // Layout partial
    fs::write(
        "demo/templates/partials/layout.hbs",
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{{title}} - Armature</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 0; padding: 0; background: #f5f5f5; }
        header { background: #333; color: white; padding: 1rem; }
        nav a { color: white; margin: 0 1rem; text-decoration: none; }
        nav a:hover { text-decoration: underline; }
        main { max-width: 1200px; margin: 2rem auto; padding: 0 1rem; }
        .card { background: white; padding: 1.5rem; margin-bottom: 1rem; border-radius: 5px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
        .badge { display: inline-block; padding: 0.25rem 0.5rem; border-radius: 3px; font-size: 0.875rem; }
        .badge-admin { background: #f44336; color: white; }
        .badge-user { background: #2196F3; color: white; }
        .badge-active { background: #4CAF50; color: white; }
        .badge-inactive { background: #9E9E9E; color: white; }
    </style>
</head>
<body>
    <header>
        <h1>📝 Armature Handlebars</h1>
        <nav>
            <a href="/">Home</a>
            <a href="/users">Users</a>
            <a href="/posts">Posts</a>
        </nav>
    </header>
    <main>
        {{{body}}}
    </main>
</body>
</html>"#
    )
    .await?;

    // Index template
    fs::write(
        "demo/templates/index.hbs",
        r#"<div class="card">
    <h1>{{title}}</h1>
    <p>{{description}}</p>

    <h2>Features Demonstrated:</h2>
    <ul>
        <li>✅ Handlebars template rendering</li>
        <li>✅ Layout partials</li>
        <li>✅ Built-in helpers (eq, upper, len, etc.)</li>
        <li>✅ Conditional rendering</li>
        <li>✅ Iteration with {{{{raw}}}}{{#each}}{{{{/raw}}}}</li>
        <li>✅ Hot-reload in dev mode</li>
    </ul>

    <h3>Examples:</h3>
    <ul>
        <li><a href="/users">View Users</a> - List all users with filtering</li>
        <li><a href="/posts">View Posts</a> - Blog post listing</li>
    </ul>
</div>"#
    )
    .await?;

    // Users list template
    fs::write(
        "demo/templates/users/list.hbs",
        r#"<div class="card">
    <h1>{{upper title}}</h1>
    <p>Total users: {{len users}}</p>

    {{#each users}}
    <div class="card">
        <h3>{{this.name}}</h3>
        <p>Email: {{this.email}}</p>
        <p>
            Role: <span class="badge badge-{{this.role}}">{{upper this.role}}</span>
            Status:
            {{#if this.active}}
                <span class="badge badge-active">ACTIVE</span>
            {{else}}
                <span class="badge badge-inactive">INACTIVE</span>
            {{/if}}
        </p>
        <a href="/users/{{this.id}}">View Details →</a>
    </div>
    {{/each}}
</div>"#
    )
    .await?;

    // User detail template
    fs::write(
        "demo/templates/users/detail.hbs",
        r#"<div class="card">
    <h1>{{title}}</h1>

    <p><strong>ID:</strong> {{user.id}}</p>
    <p><strong>Name:</strong> {{user.name}}</p>
    <p><strong>Email:</strong> {{user.email}}</p>
    <p><strong>Role:</strong> <span class="badge badge-{{user.role}}">{{upper user.role}}</span></p>
    <p><strong>Status:</strong>
        {{#if user.active}}
            <span class="badge badge-active">ACTIVE</span>
        {{else}}
            <span class="badge badge-inactive">INACTIVE</span>
        {{/if}}
    </p>

    {{#if (eq user.role "admin")}}
        <div style="background: #fff3cd; padding: 1rem; border-radius: 5px; margin-top: 1rem;">
            ⚠️ This user has administrator privileges.
        </div>
    {{/if}}

    <p style="margin-top: 2rem;">
        <a href="/users">← Back to Users</a>
    </p>
</div>"#
    )
    .await?;

    // Posts list template
    fs::write(
        "demo/templates/posts/list.hbs",
        r#"<div class="card">
    <h1>{{title}}</h1>
    <p>{{len posts}} posts available</p>

    {{#each posts}}
    <div class="card">
        <h2>{{this.title}}</h2>
        <p>{{this.content}}</p>
        <p>
            <small>By {{this.author}} | Views: {{this.views}}</small>
        </p>
        {{#if (gt this.views 1000)}}
            <span class="badge badge-active">🔥 Popular</span>
        {{/if}}
        <p><a href="/posts/{{this.id}}">Read More →</a></p>
    </div>
    {{/each}}
</div>"#
    )
    .await?;

    // Post detail template
    fs::write(
        "demo/templates/posts/detail.hbs",
        r#"<div class="card">
    <h1>{{post.title}}</h1>
    <p><small>By {{post.author}} | Views: {{post.views}}</small></p>

    {{#if (gt post.views 1000)}}
        <span class="badge badge-active">🔥 Popular Post</span>
    {{/if}}

    <div style="margin: 2rem 0;">
        {{post.content}}
    </div>

    <p style="margin-top: 2rem;">
        <a href="/posts">← Back to Posts</a>
    </p>
</div>"#
    )
    .await?;

    println!("✅ Demo templates created");

    Ok(())
}

