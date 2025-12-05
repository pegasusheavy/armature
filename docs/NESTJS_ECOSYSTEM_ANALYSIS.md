# NestJS Ecosystem Analysis & Recommendations for Armature

This document analyzes the NestJS ecosystem and provides prioritized recommendations for features to implement in Armature.

## Current Status

### ✅ Already Implemented

1. **Core Framework**
   - ✅ Dependency Injection (DI)
   - ✅ Modules
   - ✅ Controllers
   - ✅ Services/Providers
   - ✅ Decorator-style syntax (via macros)

2. **Configuration**
   - ✅ @nestjs/config equivalent (armature-config)
   - ✅ Environment variables
   - ✅ .env file support
   - ✅ Type-safe configuration

3. **GraphQL**
   - ✅ @nestjs/graphql equivalent (armature-graphql)
   - ✅ Programmatic schema building
   - ✅ Queries, mutations, subscriptions
   - ✅ DI integration

4. **Real-Time Communication**
   - ✅ WebSockets
   - ✅ Server-Sent Events (SSE)
   - ✅ Broadcasting

---

## High Priority Recommendations

### 1. 🔐 Authentication & Authorization (@nestjs/passport, @nestjs/jwt)

**Priority: HIGH**

**NestJS Example:**
```typescript
@Injectable()
export class AuthService {
  constructor(private jwtService: JwtService) {}

  async login(user: User) {
    const payload = { username: user.username, sub: user.id };
    return {
      access_token: this.jwtService.sign(payload),
    };
  }
}

@UseGuards(JwtAuthGuard)
@Get('profile')
getProfile(@Request() req) {
  return req.user;
}
```

**Armature Implementation:**
```rust
// armature-auth crate

#[injectable]
#[derive(Clone)]
struct AuthService {
    jwt_service: JwtService,
}

impl AuthService {
    async fn login(&self, user: User) -> AuthToken {
        let claims = Claims { user_id: user.id, exp: ... };
        self.jwt_service.sign(claims)
    }
}

// Usage with guards
#[controller("/api")]
#[derive(Clone)]
struct UserController {
    auth_service: AuthService,
}

impl UserController {
    #[get("/profile")]
    #[guard(JwtGuard)]  // 👈 Guard decorator
    async fn get_profile(&self, req: HttpRequest) -> Result<Json<User>, Error> {
        let user = req.user()?;
        Ok(Json(user))
    }
}
```

**Components Needed:**
- `armature-auth` crate
- JWT token generation/validation
- Password hashing (bcrypt, argon2)
- Guard system
- Session management
- OAuth2 support (optional)

---

### 2. 🛡️ Guards & Authorization

**Priority: HIGH**

**NestJS Example:**
```typescript
@Injectable()
export class RolesGuard implements CanActivate {
  canActivate(context: ExecutionContext): boolean {
    const roles = this.reflector.get('roles', context.getHandler());
    const request = context.switchToHttp().getRequest();
    return matchRoles(roles, request.user.roles);
  }
}

@Roles('admin')
@UseGuards(RolesGuard)
@Delete(':id')
deleteUser(@Param('id') id: string) {}
```

**Armature Implementation:**
```rust
// Guard trait
#[async_trait]
trait Guard {
    async fn can_activate(&self, req: &HttpRequest) -> Result<bool, Error>;
}

// Role guard
struct RoleGuard {
    required_roles: Vec<Role>,
}

#[async_trait]
impl Guard for RoleGuard {
    async fn can_activate(&self, req: &HttpRequest) -> Result<bool, Error> {
        let user = req.user()?;
        Ok(user.has_any_role(&self.required_roles))
    }
}

// Usage
impl UserController {
    #[delete("/:id")]
    #[roles("admin")]  // 👈 Decorator syntax
    async fn delete_user(&self, req: HttpRequest) -> Result<(), Error> {
        // Only admins can reach here
    }
}
```

---

### 3. 🔍 Validation & Transformation Pipes (@nestjs/class-validator)

**Priority: HIGH**

**NestJS Example:**
```typescript
export class CreateUserDto {
  @IsEmail()
  email: string;

  @MinLength(8)
  @MaxLength(20)
  password: string;

  @IsOptional()
  @IsInt()
  @Min(18)
  age?: number;
}

@Post()
create(@Body(ValidationPipe) createUserDto: CreateUserDto) {}
```

**Armature Implementation:**
```rust
// armature-validator crate

#[derive(Deserialize, Validate)]
struct CreateUserDto {
    #[validate(email)]
    email: String,

    #[validate(length(min = 8, max = 20))]
    password: String,

    #[validate(range(min = 18))]
    age: Option<u32>,
}

// Automatic validation in controller
impl UserController {
    #[post("/")]
    async fn create(&self, body: Validated<CreateUserDto>) -> Result<Json<User>, Error> {
        // body is already validated
        let dto = body.into_inner();
        Ok(Json(self.user_service.create(dto).await?))
    }
}
```

**Components Needed:**
- Validation derive macro
- Built-in validators (email, length, range, regex, etc.)
- Custom validator support
- Transformation pipes
- Error formatting

---

### 4. 📚 OpenAPI/Swagger Documentation (@nestjs/swagger)

**Priority: HIGH**

**NestJS Example:**
```typescript
@ApiTags('users')
@Controller('users')
export class UsersController {
  @ApiOperation({ summary: 'Create user' })
  @ApiResponse({ status: 201, type: User })
  @Post()
  create(@Body() createUserDto: CreateUserDto) {}
}
```

**Armature Implementation:**
```rust
// armature-openapi crate

#[controller("/users")]
#[api_tags("users")]
#[derive(Clone)]
struct UsersController {
    user_service: UserService,
}

impl UsersController {
    #[post("/")]
    #[api_operation(summary = "Create user")]
    #[api_response(status = 201, schema = "User")]
    async fn create(&self, body: Json<CreateUserDto>) -> Result<Json<User>, Error> {
        // Implementation
    }
}

// Generate OpenAPI spec
let spec = OpenApiSpec::from_application(&app);
spec.to_json_file("openapi.json")?;
```

**Components Needed:**
- OpenAPI 3.0 spec generation
- Swagger UI integration
- Schema derivation from types
- API documentation macros
- Redoc support (optional)

---

### 5. 🔄 Interceptors

**Priority: MEDIUM**

**NestJS Example:**
```typescript
@Injectable()
export class LoggingInterceptor implements NestInterceptor {
  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    console.log('Before...');
    const now = Date.now();
    return next.handle().pipe(
      tap(() => console.log(`After... ${Date.now() - now}ms`))
    );
  }
}

@UseInterceptors(LoggingInterceptor)
@Get()
findAll() {}
```

**Armature Implementation:**
```rust
#[async_trait]
trait Interceptor {
    async fn intercept(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error>;
}

struct LoggingInterceptor;

#[async_trait]
impl Interceptor for LoggingInterceptor {
    async fn intercept(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        let start = Instant::now();
        let response = next.handle(req).await?;
        let duration = start.elapsed();
        println!("Request took {:?}", duration);
        Ok(response)
    }
}

// Usage
#[get("/")]
#[interceptor(LoggingInterceptor)]
async fn find_all(&self) -> Result<Json<Vec<User>>, Error> {}
```

---

### 6. 🚦 Rate Limiting (@nestjs/throttler)

**Priority: MEDIUM**

**NestJS Example:**
```typescript
@Throttle(10, 60) // 10 requests per 60 seconds
@Get()
findAll() {}
```

**Armature Implementation:**
```rust
// armature-throttle crate

#[get("/")]
#[throttle(limit = 10, window = 60)]  // 10 requests per 60 seconds
async fn find_all(&self) -> Result<Json<Vec<User>>, Error> {}

// Or globally
let app = Application::create::<AppModule>()
    .with_throttle(ThrottleConfig {
        limit: 100,
        window: Duration::from_secs(60),
    });
```

---

### 7. 💾 Caching (@nestjs/cache-manager)

**Priority: MEDIUM**

**NestJS Example:**
```typescript
@Injectable()
export class UsersService {
  @CacheKey('users')
  @CacheTTL(60)
  async findAll(): Promise<User[]> {
    return this.userRepository.find();
  }
}
```

**Armature Implementation:**
```rust
// armature-cache crate

#[injectable]
#[derive(Clone)]
struct UserService {
    cache: CacheService,
}

impl UserService {
    #[cache(key = "users", ttl = 60)]
    async fn find_all(&self) -> Result<Vec<User>, Error> {
        // Results automatically cached
    }
}

// With Redis support
let cache = CacheService::redis("redis://localhost:6379")?;
```

---

### 8. ⏰ Task Scheduling (@nestjs/schedule)

**Priority: MEDIUM**

**NestJS Example:**
```typescript
@Injectable()
export class TasksService {
  @Cron('45 * * * * *')
  handleCron() {
    console.log('Called every 45 seconds');
  }

  @Interval(10000)
  handleInterval() {
    console.log('Called every 10 seconds');
  }
}
```

**Armature Implementation:**
```rust
// armature-schedule crate

#[injectable]
#[derive(Clone)]
struct TasksService;

impl TasksService {
    #[cron("45 * * * * *")]
    async fn handle_cron(&self) {
        println!("Called every 45 seconds");
    }

    #[interval(seconds = 10)]
    async fn handle_interval(&self) {
        println!("Called every 10 seconds");
    }
}
```

---

### 9. 🔊 Event Emitter (@nestjs/event-emitter)

**Priority: MEDIUM**

**NestJS Example:**
```typescript
// Emit event
this.eventEmitter.emit('user.created', new UserCreatedEvent());

// Listen to event
@OnEvent('user.created')
handleUserCreatedEvent(payload: UserCreatedEvent) {}
```

**Armature Implementation:**
```rust
// armature-events crate

#[injectable]
#[derive(Clone)]
struct UserService {
    event_bus: EventBus,
}

impl UserService {
    async fn create_user(&self, dto: CreateUserDto) -> Result<User, Error> {
        let user = User::new(dto);
        self.event_bus.emit(UserCreatedEvent { user: user.clone() }).await;
        Ok(user)
    }
}

#[injectable]
#[derive(Clone)]
struct NotificationService;

impl NotificationService {
    #[on_event("user.created")]
    async fn handle_user_created(&self, event: UserCreatedEvent) {
        // Send welcome email
    }
}
```

---

### 10. 🏥 Health Checks (@nestjs/terminus)

**Priority: MEDIUM**

**NestJS Example:**
```typescript
@Get('health')
@HealthCheck()
check() {
  return this.health.check([
    () => this.db.pingCheck('database'),
    () => this.http.pingCheck('api', 'https://api.example.com'),
  ]);
}
```

**Armature Implementation:**
```rust
// armature-health crate

#[get("/health")]
async fn health_check(&self) -> Result<Json<HealthStatus>, Error> {
    let status = HealthCheck::new()
        .add_check("database", self.db.ping())
        .add_check("redis", self.cache.ping())
        .add_check("api", http_ping("https://api.example.com"))
        .execute()
        .await?;

    Ok(Json(status))
}
```

---

### 11. 📨 Job Queues (@nestjs/bull)

**Priority: LOW-MEDIUM**

**NestJS Example:**
```typescript
@Processor('email')
export class EmailProcessor {
  @Process('welcome')
  async handleWelcome(job: Job) {
    await this.sendWelcomeEmail(job.data.email);
  }
}

// Add job to queue
await this.emailQueue.add('welcome', { email: 'user@example.com' });
```

**Armature Implementation:**
```rust
// armature-queue crate (using tokio + Redis)

#[processor("email")]
#[derive(Clone)]
struct EmailProcessor;

impl EmailProcessor {
    #[process("welcome")]
    async fn handle_welcome(&self, job: Job<WelcomeEmailData>) -> Result<(), Error> {
        self.send_welcome_email(&job.data.email).await
    }
}

// Add job
queue.add("email", "welcome", WelcomeEmailData {
    email: "user@example.com".to_string()
}).await?;
```

---

### 12. 🎭 Middleware

**Priority: HIGH**

**NestJS Example:**
```typescript
@Injectable()
export class LoggerMiddleware implements NestMiddleware {
  use(req: Request, res: Response, next: NextFunction) {
    console.log('Request...');
    next();
  }
}

// Apply middleware
consumer
  .apply(LoggerMiddleware)
  .forRoutes('cats');
```

**Armature Implementation:**
```rust
#[async_trait]
trait Middleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error>;
}

struct LoggerMiddleware;

#[async_trait]
impl Middleware for LoggerMiddleware {
    async fn handle(&self, req: HttpRequest, next: Next) -> Result<HttpResponse, Error> {
        println!("Request: {} {}", req.method(), req.path());
        next.handle(req).await
    }
}

// Apply to module
#[module(
    providers: [UserService],
    controllers: [UserController],
    middleware: [LoggerMiddleware]  // 👈
)]
struct UserModule;
```

---

### 13. 🗄️ Database Integration

**Priority: MEDIUM**

**Options:**
- **SQLx** (async SQL)
- **Diesel** (ORM)
- **SeaORM** (async ORM)

**NestJS Example:**
```typescript
@Entity()
export class User {
  @PrimaryGeneratedColumn()
  id: number;

  @Column()
  name: string;
}

@Injectable()
export class UsersService {
  constructor(
    @InjectRepository(User)
    private usersRepository: Repository<User>,
  ) {}
}
```

**Armature Implementation:**
```rust
// armature-orm crate (wrapper around SeaORM/SQLx)

#[derive(Model, Clone)]
#[table("users")]
struct User {
    #[primary_key]
    id: i32,
    name: String,
    email: String,
}

#[injectable]
#[derive(Clone)]
struct UserService {
    db: Database,
}

impl UserService {
    async fn find_all(&self) -> Result<Vec<User>, Error> {
        User::find().all(&self.db).await
    }
}
```

---

### 14. 🧪 Testing Utilities

**Priority: MEDIUM**

**NestJS Example:**
```typescript
describe('CatsController', () => {
  let controller: CatsController;

  beforeEach(async () => {
    const module = await Test.createTestingModule({
      controllers: [CatsController],
      providers: [CatsService],
    }).compile();

    controller = module.get<CatsController>(CatsController);
  });

  it('should return an array of cats', async () => {
    expect(await controller.findAll()).toEqual([]);
  });
});
```

**Armature Implementation:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use armature::testing::*;

    #[tokio::test]
    async fn test_find_all() {
        let test_module = TestModule::new()
            .with_controller(UserController::default())
            .with_provider(MockUserService::default())
            .build();

        let controller = test_module.get::<UserController>();
        let result = controller.find_all().await.unwrap();
        assert!(result.is_empty());
    }
}
```

---

## Implementation Priority Matrix

| Feature | Priority | Effort | Impact | NestJS Equivalent |
|---------|----------|--------|--------|-------------------|
| **Auth & JWT** | 🔴 HIGH | High | High | @nestjs/passport, @nestjs/jwt |
| **Guards** | 🔴 HIGH | Medium | High | Guards |
| **Validation Pipes** | 🔴 HIGH | Medium | High | class-validator |
| **OpenAPI/Swagger** | 🔴 HIGH | Medium | High | @nestjs/swagger |
| **Middleware** | 🔴 HIGH | Low | High | Middleware |
| **Interceptors** | 🟡 MEDIUM | Medium | Medium | Interceptors |
| **Rate Limiting** | 🟡 MEDIUM | Low | Medium | @nestjs/throttler |
| **Caching** | 🟡 MEDIUM | Medium | Medium | @nestjs/cache-manager |
| **Task Scheduling** | 🟡 MEDIUM | Medium | Medium | @nestjs/schedule |
| **Event Emitter** | 🟡 MEDIUM | Low | Medium | @nestjs/event-emitter |
| **Health Checks** | 🟡 MEDIUM | Low | Medium | @nestjs/terminus |
| **Database ORM** | 🟡 MEDIUM | High | High | @nestjs/typeorm |
| **Testing Utils** | 🟡 MEDIUM | Medium | Medium | @testing |
| **Job Queues** | 🟢 LOW | High | Medium | @nestjs/bull |
| **Microservices** | 🟢 LOW | Very High | Medium | @nestjs/microservices |
| **CQRS** | 🟢 LOW | High | Low | @nestjs/cqrs |

---

## Recommended Implementation Order

### Phase 1: Security & Validation (Weeks 1-3)
1. **Middleware system**
2. **Guards & Authorization**
3. **Authentication (JWT)**
4. **Validation Pipes**

### Phase 2: Documentation & Developer Experience (Weeks 4-5)
5. **OpenAPI/Swagger generation**
6. **Testing utilities**

### Phase 3: Performance & Scalability (Weeks 6-8)
7. **Interceptors**
8. **Rate Limiting**
9. **Caching**

### Phase 4: Background Processing (Weeks 9-10)
10. **Event Emitter**
11. **Task Scheduling**

### Phase 5: Monitoring & Maintenance (Week 11)
12. **Health Checks**

### Phase 6: Data Layer (Weeks 12-14)
13. **Database Integration (ORM wrapper)**

### Phase 7: Advanced Features (Future)
14. **Job Queues**
15. **Microservices**
16. **CQRS**

---

## Rust-Specific Advantages

While implementing NestJS patterns, we can leverage Rust's unique features:

1. **Zero-Cost Abstractions** - No runtime overhead
2. **Memory Safety** - No null pointer exceptions
3. **Fearless Concurrency** - Safe async/await
4. **Strong Type System** - Better compile-time guarantees
5. **Performance** - Much faster than Node.js
6. **No GC Pauses** - Predictable performance

---

## Conclusion

The Armature framework has a strong foundation with DI, modules, configuration, and GraphQL already implemented. By systematically adding the features above, starting with security and validation, we can create a comprehensive Rust framework that rivals NestJS while leveraging Rust's performance and safety advantages.

**Next Steps:**
1. Start with Phase 1 (Security & Validation)
2. Create individual crates for each major feature
3. Maintain the decorator-style API for consistency
4. Build comprehensive examples for each feature
5. Document everything thoroughly

This roadmap will make Armature the **"NestJS of Rust"** - a complete, production-ready framework for building scalable server-side applications.

