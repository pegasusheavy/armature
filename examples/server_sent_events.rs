#![allow(
    dead_code,
    unused_imports,
    clippy::default_constructed_unit_structs,
    clippy::needless_borrow,
    clippy::unnecessary_lazy_evaluations
)]
// Server-Sent Events (SSE) example

use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StockPrice {
    symbol: String,
    price: f64,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct NewsItem {
    title: String,
    content: String,
    timestamp: u64,
}

/// Stock ticker service using SSE
#[injectable]
#[derive(Clone, Default)]
struct StockTickerService;

impl StockTickerService {
    fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "service": "Stock Ticker",
            "status": "running",
            "symbols": ["AAPL", "GOOGL", "MSFT", "AMZN"]
        })
    }
}

/// News service using SSE
#[injectable]
#[derive(Clone, Default)]
struct NewsService;

impl NewsService {
    fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "service": "News Feed",
            "status": "running"
        })
    }
}

/// SSE controller
#[controller("/events")]
#[derive(Default, Clone)]
struct EventsController;

impl EventsController {
    #[get("/stats")]
    async fn get_stats() -> Result<Json<serde_json::Value>, Error> {
        let stock_service = StockTickerService::default();
        let news_service = NewsService::default();

        Ok(Json(serde_json::json!({
            "stock_ticker": stock_service.get_stats(),
            "news": news_service.get_stats()
        })))
    }

    #[get("/stocks")]
    async fn get_stocks_stream() -> Result<HttpResponse, Error> {
        // In a full implementation, this would return an SSE stream
        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/event-stream".to_string())
            .with_header("Cache-Control".to_string(), "no-cache".to_string())
            .with_body(b"event: connected\ndata: {\"status\": \"connected\"}\n\n".to_vec()))
    }

    #[get("/news")]
    async fn get_news_stream() -> Result<HttpResponse, Error> {
        // In a full implementation, this would return an SSE stream
        Ok(HttpResponse::ok()
            .with_header("Content-Type".to_string(), "text/event-stream".to_string())
            .with_header("Cache-Control".to_string(), "no-cache".to_string())
            .with_body(b"event: connected\ndata: {\"status\": \"connected\"}\n\n".to_vec()))
    }
}

#[module(
    providers: [StockTickerService, NewsService],
    controllers: [EventsController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("📡 Armature Server-Sent Events Example");
    println!("=======================================\n");

    println!("Available endpoints:");
    println!("  GET /events/stocks - Stock price updates (SSE stream)");
    println!("  GET /events/news   - News updates (SSE stream)");
    println!("  GET /events/stats  - Get service statistics");

    println!("\n💡 Usage with curl:");
    println!("  curl -N http://localhost:3006/events/stocks");
    println!("  curl -N http://localhost:3006/events/news");

    println!("\n💡 Usage in JavaScript:");
    println!("  const source = new EventSource('http://localhost:3006/events/stocks');");
    println!("  source.addEventListener('price_update', (e) => {{");
    println!("    const data = JSON.parse(e.data);");
    println!("    console.log('Stock price:', data);");
    println!("  }});");
    println!();

    let app = Application::create::<AppModule>().await;

    if let Err(e) = app.listen(3006).await {
        eprintln!("Server error: {}", e);
    }
}
