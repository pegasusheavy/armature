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
    #[get("")]
    async fn list() -> Result<Json<Vec<Book>>, Error> {
        let service = BookService::default();
        let books = service.get_all_books();
        Ok(Json(books))
    }

    #[get("/:id")]
    async fn get(req: HttpRequest) -> Result<Json<Book>, Error> {
        let id_str = req
            .param("id")
            .ok_or_else(|| Error::Validation("Missing id".to_string()))?;
        let id: u32 = id_str
            .parse()
            .map_err(|_| Error::Validation("Invalid id".to_string()))?;

        let service = BookService::default();
        let book = service
            .get_all_books()
            .into_iter()
            .find(|b| b.id == id)
            .ok_or_else(|| Error::RouteNotFound("Book not found".to_string()))?;

        Ok(Json(book))
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

    println!("Available routes:");
    println!("  GET /books     - List all books");
    println!("  GET /books/:id - Get book by ID\n");

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3004).await {
        eprintln!("Server error: {}", e);
    }
}
