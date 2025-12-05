// Example showing automatic DI using Application::create()

use armature::prelude::*;
use serde::{Deserialize, Serialize};

// ========== Models ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Book {
    id: u32,
    title: String,
    author: String,
}

// ========== Services ==========

#[injectable]
#[derive(Default, Clone)]
struct LoggerService;

impl LoggerService {
    fn log(&self, message: &str) {
        println!("[LOG] {}", message);
    }
}

#[injectable]
#[derive(Default, Clone)]
struct BookService {
    logger: LoggerService,
}

impl BookService {
    fn get_all_books(&self) -> Vec<Book> {
        self.logger.log("Fetching all books");
        vec![
            Book {
                id: 1,
                title: "The Rust Programming Language".to_string(),
                author: "Steve Klabnik".to_string(),
            },
            Book {
                id: 2,
                title: "Programming Rust".to_string(),
                author: "Jim Blandy".to_string(),
            },
        ]
    }
}

// ========== Controllers ==========

#[controller("/books")]
#[derive(Default, Clone)]
struct BookController {
    book_service: BookService,
}

impl BookController {
    fn list(&self) -> Result<Json<Vec<Book>>, Error> {
        let books = self.book_service.get_all_books();
        Ok(Json(books))
    }
}

// ========== Module ==========

#[module(
    providers: [LoggerService, BookService],
    controllers: [BookController]
)]
#[derive(Default)]
struct AppModule;

// ========== Main ==========

#[tokio::main]
async fn main() {
    println!("📚 Armature Automatic DI Example");
    println!("=================================\n");

    // This would work once full DI is integrated:
    // let app = Application::create::<AppModule>();

    // For now, demonstrate the DI pattern manually:
    let app = setup_with_di();

    println!("Available routes:");
    println!("  GET /books - List all books\n");

    if let Err(e) = app.listen(3004).await {
        eprintln!("Server error: {}", e);
    }
}

fn setup_with_di() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Step 1: Register services
    println!("Registering services:");
    let logger = LoggerService::default();
    container.register(logger.clone());
    println!("  ✓ LoggerService");

    let book_service = BookService { logger };
    container.register(book_service.clone());
    println!("  ✓ BookService");

    // Step 2: Create controller with injected dependencies
    println!("\nCreating controllers:");
    let book_controller = BookController {
        book_service: book_service.clone(),
    };
    println!("  ✓ BookController (with BookService injected)");

    // Step 3: Register routes
    println!("\nRegistering routes:");
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/books".to_string(),
        handler: std::sync::Arc::new(move |_req| {
            let ctrl = book_controller.clone();
            Box::pin(async move { ctrl.list().and_then(|j| j.into_response()) })
        }),
    });
    println!("  ✓ GET /books\n");

    Application {
        container,
        router: std::sync::Arc::new(router),
    }
}
