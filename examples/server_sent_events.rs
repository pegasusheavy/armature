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
#[derive(Clone)]
struct StockTickerService {
    broadcaster: Arc<SseBroadcaster>,
}

impl Default for StockTickerService {
    fn default() -> Self {
        let broadcaster = Arc::new(SseBroadcaster::new());

        // Start broadcasting stock prices
        let broadcaster_clone = broadcaster.clone();
        tokio::spawn(async move {
            let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN"];
            let mut interval = tokio::time::interval(Duration::from_secs(2));

            loop {
                interval.tick().await;

                for symbol in &symbols {
                    let price = StockPrice {
                        symbol: symbol.to_string(),
                        price: 100.0 + (rand::random::<f64>() * 50.0),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };

                    let event = ServerSentEvent::with_event(
                        "price_update".to_string(),
                        serde_json::to_string(&price).unwrap(),
                    );

                    let _ = broadcaster_clone.broadcast(event).await;
                }
            }
        });

        // Start keep-alive
        let broadcaster_clone = broadcaster.clone();
        broadcaster_clone.start_keep_alive(Duration::from_secs(15));

        Self { broadcaster }
    }
}

impl StockTickerService {
    async fn get_client_count(&self) -> usize {
        self.broadcaster.client_count().await
    }

    async fn register_client(
        &self,
    ) -> tokio_stream::wrappers::ReceiverStream<Result<String, Error>> {
        self.broadcaster.register().await
    }
}

/// News service using SSE
#[injectable]
#[derive(Clone)]
struct NewsService {
    broadcaster: Arc<SseBroadcaster>,
}

impl Default for NewsService {
    fn default() -> Self {
        let broadcaster = Arc::new(SseBroadcaster::new());

        // Simulate news updates
        let broadcaster_clone = broadcaster.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));
            let mut counter = 0;

            loop {
                interval.tick().await;
                counter += 1;

                let news = NewsItem {
                    title: format!("Breaking News #{}", counter),
                    content: format!("This is important news update number {}", counter),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };

                let event = ServerSentEvent::with_event(
                    "news".to_string(),
                    serde_json::to_string(&news).unwrap(),
                );

                let _ = broadcaster_clone.broadcast(event).await;
            }
        });

        Self { broadcaster }
    }
}

/// SSE controller
#[controller("/events")]
#[derive(Default, Clone)]
struct EventsController {
    stock_ticker: StockTickerService,
    news_service: NewsService,
}

impl EventsController {
    async fn get_stock_stats(&self) -> Result<Json<serde_json::Value>, Error> {
        let count = self.stock_ticker.get_client_count().await;
        Ok(Json(serde_json::json!({
            "subscribers": count
        })))
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

    let app = create_sse_app();

    println!("Available endpoints:");
    println!("  GET /events/stocks - Stock price updates (SSE stream)");
    println!("  GET /events/news   - News updates (SSE stream)");
    println!("  GET /events/stats  - Get subscriber statistics");

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

    if let Err(e) = app.listen(3006).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_sse_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let stock_ticker = StockTickerService::default();
    let news_service = NewsService::default();

    container.register(stock_ticker.clone());
    container.register(news_service.clone());

    // Create controller
    let events_controller = EventsController {
        stock_ticker: stock_ticker.clone(),
        news_service: news_service.clone(),
    };

    // Note: In a full implementation, SSE routes would return streaming responses
    // For now, we demonstrate the structure

    // Stats endpoint
    let controller_clone = events_controller.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/events/stats".to_string(),
        handler: Arc::new(move |_req| {
            let ctrl = controller_clone.clone();
            Box::pin(async move { ctrl.get_stock_stats().await?.into_response() })
        }),
    });

    Application::new(container, router)
}

// Simple random number generator (avoiding external dependency)
mod rand {
    use std::cell::Cell;

    thread_local! {
        static SEED: Cell<u64> = Cell::new(0x123456789abcdef0);
    }

    pub fn random<T: From<f64>>() -> T {
        SEED.with(|seed| {
            let mut s = seed.get();
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            seed.set(s);
            ((s as f64) / (u64::MAX as f64)).into()
        })
    }
}
