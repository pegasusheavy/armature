use armature::prelude::*;
use armature::armature_opentelemetry::*;

// Example service
#[derive(Clone)]
#[injectable]
struct UserService;

impl UserService {
    fn get_user(&self, id: u64) -> Result<String, Error> {
        // Add span attribute
        span_attribute!("user.id", id.to_string());

        // Record span event
        span_event!("fetching_user", "user.id" => id.to_string());

        Ok(format!("User {}", id))
    }
}

// Example controller
#[controller("/users")]
struct UserController {
    user_service: UserService,
}

impl UserController {
    #[get("/:id")]
    async fn get_user(&self, #[Param("id")] id: u64) -> Result<Json<serde_json::Value>, Error> {
        let user = self.user_service.get_user(id)?;

        Ok(Json(serde_json::json!({
            "user": user
        })))
    }

    #[get("/")]
    async fn list_users(&self) -> Result<Json<serde_json::Value>, Error> {
        Ok(Json(serde_json::json!({
            "users": ["Alice", "Bob", "Charlie"]
        })))
    }
}

// Application module
#[module]
struct AppModule;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔭 Starting Armature with OpenTelemetry...");

    // Initialize OpenTelemetry with OTLP exporter
    // Note: Requires an OTLP collector running on localhost:4317
    // You can use Jaeger all-in-one: docker run -p 16686:16686 -p 4317:4317 jaegertracing/all-in-one:latest
    let telemetry = TelemetryBuilder::new("armature-example")
        .with_version("1.0.0")
        .with_environment("development")
        .with_otlp_endpoint("http://localhost:4317")
        .with_tracing()
        .with_metrics()
        .with_sampling_ratio(1.0) // Sample all traces
        .with_attribute("team", "platform")
        .build()
        .await?;

    println!("✅ OpenTelemetry initialized");
    println!("📊 Tracing: Enabled");
    println!("📈 Metrics: Enabled");
    println!("🔗 OTLP Endpoint: http://localhost:4317");
    println!("🌐 View traces at: http://localhost:16686 (if using Jaeger)");

    // Create application
    let (container, mut router) = Application::create::<AppModule>().await?;

    // Add OpenTelemetry middleware for automatic instrumentation
    let app = Application::new(container, router).with_middleware(telemetry.middleware());

    println!("🚀 Server running on http://localhost:3000");
    println!("📝 Try: curl http://localhost:3000/users");
    println!("📝 Try: curl http://localhost:3000/users/42");
    println!("⏹️  Press Ctrl+C to shutdown");

    // Start server
    tokio::select! {
        _ = app.listen("0.0.0.0:3000") => {},
        _ = tokio::signal::ctrl_c() => {
            println!("\n⏹️  Shutting down...");
        }
    }

    // Gracefully shutdown telemetry
    println!("🔄 Flushing telemetry data...");
    telemetry.shutdown().await?;
    println!("✅ Telemetry shutdown complete");

    Ok(())
}

