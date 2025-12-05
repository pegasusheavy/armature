# Dependency Injection Guide

This guide explains how dependency injection works in Armature and how to use it effectively.

## Overview

Armature provides a complete dependency injection system inspired by Angular. Services are automatically injected into controllers based on their field types, enabling loose coupling and testability.

## Core Concepts

### 1. Injectable Services

Mark a struct with `#[injectable]` to make it available for injection:

```rust
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    connection_string: String,
}
```

**Requirements:**
- Must implement `Default` (for automatic instantiation)
- Must implement `Clone` (for sharing across the application)
- Must be `Send + Sync + 'static` (for thread safety)

### 2. Service Dependencies

Services can depend on other services by declaring them as fields:

```rust
#[injectable]
#[derive(Default, Clone)]
struct UserService {
    database: DatabaseService,  // Will be auto-injected
    logger: LoggerService,       // Will be auto-injected
}
```

### 3. Controllers with DI

Controllers automatically receive injected services:

```rust
#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,  // Automatically injected!
}

impl UserController {
    // Methods can now use self.user_service
    fn get_users(&self) -> Result<Json<Vec<User>>, Error> {
        let users = self.user_service.find_all();
        Ok(Json(users))
    }
}
```

## How It Works

### Registration Order

The framework automatically handles dependency registration in the correct order:

1. **Imported modules** are registered first (depth-first)
2. **Providers** (services) are registered in declaration order
3. **Controllers** are instantiated with resolved dependencies
4. **Routes** are registered for each controller

### Dependency Resolution

When a controller is created:

1. The framework inspects the controller's fields
2. For each field, it resolves the service from the DI container
3. The controller is constructed with all dependencies injected
4. The controller instance is cached for reuse

### Container Lifecycle

- Services are **singletons** by default
- Once created, the same instance is shared across the application
- This ensures efficient resource usage (e.g., database connections)

## Usage Examples

### Example 1: Simple Service Injection

```rust
use armature::prelude::*;

// Service with no dependencies
#[injectable]
#[derive(Default, Clone)]
struct ConfigService {
    api_url: String,
}

// Controller using the service
#[controller("/api")]
#[derive(Default, Clone)]
struct ApiController {
    config: ConfigService,
}

impl ApiController {
    #[get("/info")]
    async fn info(&self) -> Result<Json<String>, Error> {
        Ok(Json(self.config.api_url.clone()))
    }
}

#[module(
    providers: [ConfigService],
    controllers: [ApiController]
)]
#[derive(Default)]
struct AppModule;
```

### Example 2: Service Chain

```rust
// Level 1: Base service
#[injectable]
#[derive(Default, Clone)]
struct LoggerService;

// Level 2: Service depending on Logger
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    logger: LoggerService,
}

// Level 3: Service depending on Database
#[injectable]
#[derive(Default, Clone)]
struct UserService {
    database: DatabaseService,
}

// Level 4: Controller depending on UserService
#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,
}

#[module(
    providers: [LoggerService, DatabaseService, UserService],
    controllers: [UserController]
)]
#[derive(Default)]
struct AppModule;
```

The framework ensures all dependencies are resolved in the correct order.

### Example 3: Multiple Dependencies

```rust
#[injectable]
#[derive(Default, Clone)]
struct AuthService;

#[injectable]
#[derive(Default, Clone)]
struct CacheService;

#[injectable]
#[derive(Default, Clone)]
struct EmailService;

#[injectable]
#[derive(Default, Clone)]
struct UserService {
    auth: AuthService,
    cache: CacheService,
    email: EmailService,
}

#[controller("/users")]
#[derive(Default, Clone)]
struct UserController {
    user_service: UserService,
    auth_service: AuthService,  // Can inject same service multiple times
}
```

## Module System

### Provider Declaration

Providers must be declared in the module:

```rust
#[module(
    providers: [ServiceA, ServiceB, ServiceC],
    controllers: [ControllerX, ControllerY]
)]
```

**Order matters for providers:**
- List services with no dependencies first
- Then list services that depend on earlier services
- The framework registers them in declaration order

### Module Imports

Modules can import other modules to access their services:

```rust
#[module(
    providers: [SharedService],
    exports: [SharedService]  // Make available to importers
)]
#[derive(Default)]
struct SharedModule;

#[module(
    providers: [UserService],
    controllers: [UserController],
    imports: [SharedModule]  // Import shared services
)]
#[derive(Default)]
struct UserModule;
```

## Advanced Patterns

### Constructor Injection

The generated `new_with_di` method is automatically called:

```rust
// Generated automatically by #[controller]
impl UserController {
    pub fn new_with_di(container: &Container) -> Result<Self, Error> {
        Ok(Self {
            user_service: (*container.resolve::<UserService>()?).clone(),
        })
    }
}
```

### Manual DI (for advanced use cases)

You can manually work with the container:

```rust
let container = Container::new();

// Register a service
container.register(MyService::default());

// Resolve a service
let service = container.resolve::<MyService>()?;
```

### Testing with DI

DI makes testing easier by allowing mock injection:

```rust
#[cfg(test)]
mod tests {
    #[injectable]
    #[derive(Default, Clone)]
    struct MockDatabaseService {
        // Mock implementation
    }

    #[test]
    fn test_controller() {
        let container = Container::new();
        container.register(MockDatabaseService::default());
        
        let controller = UserController::new_with_di(&container).unwrap();
        // Test controller with mock dependencies
    }
}
```

## Best Practices

### 1. Keep Services Stateless

Services should be stateless or have immutable state:

```rust
// Good: Stateless
#[injectable]
#[derive(Default, Clone)]
struct UserService {
    db: DatabaseService,  // Shared connection pool
}

// Avoid: Mutable state
#[injectable]
#[derive(Default, Clone)]
struct CounterService {
    count: i32,  // This won't work as expected with Clone
}
```

### 2. Use Descriptive Names

```rust
// Good
#[injectable]
struct UserAuthenticationService;

// Avoid
#[injectable]
struct Service1;
```

### 3. Minimize Dependencies

Keep the dependency graph shallow:

```rust
// Good: 2-3 dependencies max
#[injectable]
struct UserService {
    database: DatabaseService,
    cache: CacheService,
}

// Avoid: Too many dependencies (consider refactoring)
#[injectable]
struct GodService {
    dep1: Service1,
    dep2: Service2,
    // ... 10 more dependencies
}
```

### 4. Interface Segregation

Create focused services with single responsibilities:

```rust
// Good: Focused services
#[injectable]
struct UserRepository;  // Data access

#[injectable]
struct UserValidator;  // Validation logic

#[injectable]
struct UserNotifier;   // Notifications

// Avoid: God object
#[injectable]
struct UserEverything;  // Does everything
```

## Troubleshooting

### "Provider not found" Error

**Cause:** Service not registered in module or wrong type.

**Solution:** Ensure the service is in the `providers` array:

```rust
#[module(
    providers: [MyService],  // Must be listed here!
    controllers: [MyController]
)]
```

### Circular Dependencies

**Cause:** Service A depends on B, B depends on A.

**Solution:** Refactor to break the cycle:

```rust
// Bad: Circular dependency
struct ServiceA { b: ServiceB }
struct ServiceB { a: ServiceA }  // Circular!

// Good: Extract shared dependency
struct ServiceA { shared: SharedService }
struct ServiceB { shared: SharedService }
struct SharedService { /* shared logic */ }
```

### Clone Not Implemented

**Cause:** Service doesn't implement `Clone`.

**Solution:** Add `#[derive(Clone)]` or implement it manually:

```rust
#[injectable]
#[derive(Default, Clone)]  // Add Clone here
struct MyService;
```

## Performance Considerations

### Singleton Pattern

- Services are created once and reused
- No performance overhead after initial creation
- Thread-safe through `Arc` internally

### Clone Overhead

- `Clone` on services is usually cheap (clones Arc pointers)
- For expensive resources, use Arc/Rc internally:

```rust
#[injectable]
#[derive(Default, Clone)]
struct DatabaseService {
    pool: Arc<ConnectionPool>,  // Cheap to clone
}
```

## Future Enhancements

Planned features for the DI system:

- [ ] `@Scope` decorator for request-scoped services
- [ ] `@Factory` for custom instantiation logic
- [ ] `@Lazy` for lazy-loaded services
- [ ] Interface-based injection with traits
- [ ] Conditional providers
- [ ] Provider configuration

## Comparison with Other Frameworks

### vs Spring (Java)
- Similar `@Injectable` / `@Service` concepts
- Similar `@Controller` pattern
- No XML configuration needed

### vs Angular (TypeScript)
- Nearly identical decorator syntax
- Same module system
- Constructor injection works similarly

### vs Actix-web (Rust)
- More explicit DI (vs implicit Data extractors)
- Compile-time safety
- Better testability

## Summary

Armature's DI system provides:

✅ **Automatic injection** based on field types  
✅ **Type-safe** resolution at compile time  
✅ **Modular** organization with imports/exports  
✅ **Testable** through dependency injection  
✅ **Performant** with singleton pattern  
✅ **Familiar** syntax for Angular/Spring developers  

The DI system is the foundation of Armature, enabling clean, maintainable, and testable code.

