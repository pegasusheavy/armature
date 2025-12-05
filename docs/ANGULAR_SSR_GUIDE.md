# Angular SSR Guide

This guide explains how to use Angular Server-Side Rendering (SSR) with Armature, similar to NestJS's Angular Universal integration.

## Overview

Armature provides seamless integration with Angular Universal for server-side rendering, allowing you to:
- Render Angular applications on the server
- Serve static assets efficiently
- Mix Angular routes with API endpoints
- Benefit from SEO and faster initial page loads

## Features

✅ **Angular Universal Support** - Full SSR integration
✅ **Static File Serving** - Efficient asset delivery
✅ **API Routes** - Mix Angular and REST APIs
✅ **Caching** - Optional page caching
✅ **Security** - Path traversal protection
✅ **Hot Module Replacement** - Development mode support

## Installation

Add the Angular feature to your `Cargo.toml`:

```toml
[dependencies]
armature = { version = "0.1", features = ["angular"] }
armature-angular = "0.1"
```

## Angular Setup

### 1. Create Angular Application

```bash
# Create a new Angular app
ng new my-app
cd my-app

# Add Angular Universal (SSR)
ng add @angular/ssr
```

### 2. Build for Production

```bash
# Build both browser and server bundles
ng build
ng run my-app:server
```

This creates:
```
dist/my-app/
├── browser/          # Client-side bundle
│   ├── index.html
│   ├── main.js
│   ├── styles.css
│   └── assets/
└── server/           # Server-side bundle
    └── main.js       # Angular Universal entry point
```

## Armature Integration

### Basic Setup

```rust
use armature::prelude::*;
use armature_angular::{AngularConfig, AngularService};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Configure Angular SSR
    let config = AngularConfig::new()
        .with_node_path(PathBuf::from("node"))
        .with_server_bundle(PathBuf::from("dist/my-app/server/main.js"))
        .with_browser_dist(PathBuf::from("dist/my-app/browser"))
        .exclude_route("/api".to_string());

    // Create Angular service
    let angular_service = AngularService::new(config).unwrap();

    // Create application with Angular SSR
    let app = create_app_with_angular(angular_service);

    app.listen(3000).await.unwrap();
}
```

### Configuration Options

```rust
let config = AngularConfig::new()
    // Node.js executable path
    .with_node_path(PathBuf::from("/usr/bin/node"))

    // Server bundle (Angular Universal)
    .with_server_bundle(PathBuf::from("dist/my-app/server/main.js"))

    // Browser distribution folder
    .with_browser_dist(PathBuf::from("dist/my-app/browser"))

    // Exclude routes from SSR (e.g., API routes)
    .exclude_route("/api".to_string())
    .exclude_route("/graphql".to_string())

    // Enable caching (TTL in seconds)
    .with_cache(true, 300)

    // Render timeout (milliseconds)
    .with_timeout(5000);
```

## Routing Setup

### Complete Example

```rust
use armature::prelude::*;
use armature_angular::{AngularService, RenderOptions};

fn setup_routes(router: &mut Router, angular: AngularService) {
    // 1. API Routes (excluded from SSR)
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/api/data".to_string(),
        handler: Arc::new(|_req| {
            Box::pin(async {
                let data = serde_json::json!({"message": "Hello!"});
                HttpResponse::ok()
                    .with_header("Content-Type".into(), "application/json".into())
                    .with_json(&data)
            })
        }),
    });

    // 2. Static Assets (/assets/*, *.js, *.css)
    let static_angular = angular.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/assets/*".to_string(),
        handler: Arc::new(move |req| {
            let service = static_angular.clone();
            Box::pin(async move {
                let path = req.path();
                match service.serve_static(path).await {
                    Ok(content) => {
                        let content_type = get_content_type(path);
                        Ok(HttpResponse::ok()
                            .with_header("Content-Type".into(), content_type)
                            .with_body(content))
                    }
                    Err(_) => Ok(HttpResponse::not_found()
                        .with_body(b"Not found".to_vec())),
                }
            })
        }),
    });

    // 3. SSR for Angular Routes
    let ssr_angular = angular.clone();
    router.add_route(Route {
        method: HttpMethod::GET,
        path: "/*".to_string(),
        handler: Arc::new(move |req| {
            let service = ssr_angular.clone();
            Box::pin(async move {
                let path = req.path();

                // Check if should render server-side
                if !service.should_render(path) {
                    // Serve as static file
                    return match service.serve_static(path).await {
                        Ok(content) => Ok(HttpResponse::ok().with_body(content)),
                        Err(_) => Ok(HttpResponse::not_found()
                            .with_body(b"Not found".to_vec())),
                    };
                }

                // Perform SSR
                let options = RenderOptions::new(path.to_string());
                match service.render(path, options).await {
                    Ok(html) => Ok(HttpResponse::ok()
                        .with_header("Content-Type".into(), "text/html; charset=utf-8".into())
                        .with_body(html.into_bytes())),
                    Err(e) => {
                        eprintln!("SSR error: {}", e);
                        // Fallback to client-side rendering
                        match service.serve_static("/index.html").await {
                            Ok(content) => Ok(HttpResponse::ok()
                                .with_header("Content-Type".into(), "text/html".into())
                                .with_body(content)),
                            Err(_) => Ok(HttpResponse::internal_server_error()
                                .with_body(b"Server error".to_vec())),
                        }
                    }
                }
            })
        }),
    });
}
```

## Module Integration

### With DI

```rust
#[injectable]
#[derive(Clone)]
struct AngularService {
    // Injected automatically
}

#[controller("/api")]
#[derive(Clone)]
struct ApiController {
    data_service: DataService,
}

impl ApiController {
    #[get("/data")]
    async fn get_data(&self) -> Result<Json<Data>, Error> {
        Ok(Json(self.data_service.get_data()))
    }
}

#[module(
    providers: [AngularService, DataService],
    controllers: [ApiController]
)]
struct AppModule;

#[tokio::main]
async fn main() {
    let app = Application::create::<AppModule>();
    app.listen(3000).await.unwrap();
}
```

## Angular Configuration

### Angular App Module (app.config.server.ts)

```typescript
import { ApplicationConfig, mergeApplicationConfig } from '@angular/core';
import { provideServerRendering } from '@angular/platform-server';
import { appConfig } from './app.config';

const serverConfig: ApplicationConfig = {
  providers: [
    provideServerRendering()
  ]
};

export const config = mergeApplicationConfig(appConfig, serverConfig);
```

### Server Entry Point (main.server.ts)

```typescript
import { bootstrapApplication } from '@angular/platform-browser';
import { AppComponent } from './app/app.component';
import { config } from './app/app.config.server';

const bootstrap = () => bootstrapApplication(AppComponent, config);

export default bootstrap;
```

## API Integration

### Calling Rust APIs from Angular

```typescript
// Angular service
@Injectable({
  providedIn: 'root'
})
export class DataService {
  constructor(private http: HttpClient) {}

  getData(): Observable<any> {
    return this.http.get('/api/data');
  }
}

// Component
export class AppComponent implements OnInit {
  data: any;

  constructor(private dataService: DataService) {}

  ngOnInit() {
    this.dataService.getData().subscribe(
      data => this.data = data
    );
  }
}
```

### Transfer State (Avoid Duplicate Requests)

```typescript
import { TransferState, makeStateKey } from '@angular/platform-browser';

const DATA_KEY = makeStateKey<any>('data');

@Component({...})
export class AppComponent implements OnInit {
  constructor(
    private dataService: DataService,
    private transferState: TransferState,
    @Inject(PLATFORM_ID) private platformId: any
  ) {}

  ngOnInit() {
    // Check if data is already loaded (SSR)
    const cachedData = this.transferState.get(DATA_KEY, null);

    if (cachedData) {
      this.data = cachedData;
    } else {
      this.dataService.getData().subscribe(data => {
        this.data = data;

        // Store for client hydration
        if (isPlatformServer(this.platformId)) {
          this.transferState.set(DATA_KEY, data);
        }
      });
    }
  }
}
```

## Performance Optimization

### Caching

Enable caching for frequently accessed pages:

```rust
let config = AngularConfig::new()
    .with_cache(true, 300); // Cache for 5 minutes

let angular_service = AngularService::new(config)?;
```

### Prerendering

For completely static pages:

```bash
# Angular
ng run my-app:prerender

# Or use static site generation
ng build --configuration production
```

### Lazy Loading

```typescript
// Angular routes with lazy loading
const routes: Routes = [
  {
    path: 'admin',
    loadChildren: () => import('./admin/admin.module').then(m => m.AdminModule)
  }
];
```

## Development Workflow

### Development Mode

```bash
# Terminal 1: Angular dev server
ng serve

# Terminal 2: Armature server (proxy to Angular)
cargo run --example angular_ssr --features angular
```

### Production Build

```bash
# Build Angular
ng build --configuration production
ng run my-app:server:production

# Build Armature
cargo build --release --features angular

# Run
./target/release/my-app
```

## Deployment

### Docker

```dockerfile
# Multi-stage build
FROM node:20 AS angular-build
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build:ssr

FROM rust:1.75 AS rust-build
WORKDIR /app
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release --features angular

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y nodejs ca-certificates
COPY --from=angular-build /app/dist ./dist
COPY --from=rust-build /app/target/release/my-app ./
EXPOSE 3000
CMD ["./my-app"]
```

## Troubleshooting

### SSR Not Working

1. **Check paths**:
   ```rust
   // Ensure paths exist
   let config = AngularConfig::new()
       .with_server_bundle(PathBuf::from("dist/my-app/server/main.js"))
       .with_browser_dist(PathBuf::from("dist/my-app/browser"));
   ```

2. **Check Node.js**:
   ```bash
   node --version  # Should be v18+
   ```

3. **Check Angular build**:
   ```bash
   ls dist/my-app/server/main.js   # Should exist
   ls dist/my-app/browser/index.html  # Should exist
   ```

### Performance Issues

1. **Enable caching**:
   ```rust
   .with_cache(true, 600)  // Cache for 10 minutes
   ```

2. **Increase timeout**:
   ```rust
   .with_timeout(10000)  // 10 seconds
   ```

3. **Use CDN** for static assets

## Comparison with NestJS

### NestJS

```typescript
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';
import { NestExpressApplication } from '@nestjs/platform-express';

async function bootstrap() {
  const app = await NestFactory.create<NestExpressApplication>(AppModule);

  // Serve Angular Universal
  app.setViewEngine('html');
  app.useStaticAssets('dist/browser');

  await app.listen(3000);
}
```

### Armature

```rust
use armature::prelude::*;
use armature_angular::{AngularConfig, AngularService};

#[tokio::main]
async fn main() {
    let config = AngularConfig::new()
        .with_browser_dist(PathBuf::from("dist/browser"))
        .with_server_bundle(PathBuf::from("dist/server/main.js"));

    let angular = AngularService::new(config).unwrap();
    let app = create_app_with_angular(angular);

    app.listen(3000).await.unwrap();
}
```

## Best Practices

1. **Separate API Routes** - Use `/api/*` prefix for REST APIs
2. **Enable Caching** - Cache rendered pages in production
3. **Transfer State** - Avoid duplicate API calls
4. **Lazy Loading** - Split Angular bundles for faster loads
5. **CDN for Assets** - Serve static files from CDN in production
6. **Error Handling** - Fallback to client-side rendering on errors
7. **Security** - Validate and sanitize all inputs
8. **Monitoring** - Track SSR performance and errors

## Summary

Armature's Angular SSR support provides:

✅ **Full SSR Integration** - Angular Universal support
✅ **Performance** - Much faster than NestJS/Node.js
✅ **Type Safety** - Rust's strong type system
✅ **Memory Safety** - No memory leaks or crashes
✅ **DI Integration** - Seamless service injection
✅ **Production Ready** - Caching, error handling, security

For complete examples, see `examples/angular_ssr.rs` in the repository.

