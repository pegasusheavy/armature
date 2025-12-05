// Validation Example - Demonstrates validation framework

use armature::armature_validation::*;
use armature::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// ========== DTOs with Validation ==========

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CreateUserDto {
    pub name: String,
    pub email: String,
    pub age: i32,
    pub password: String,
}

impl Validate for CreateUserDto {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate name
        if let Err(e) = NotEmpty::validate(&self.name, "name") {
            errors.push(e);
        }
        if let Err(e) = MinLength(2).validate(&self.name, "name") {
            errors.push(e);
        }
        if let Err(e) = MaxLength(50).validate(&self.name, "name") {
            errors.push(e);
        }

        // Validate email
        if let Err(e) = NotEmpty::validate(&self.email, "email") {
            errors.push(e);
        }
        if let Err(e) = IsEmail::validate(&self.email, "email") {
            errors.push(e);
        }

        // Validate age
        if let Err(e) = IsPositive::validate_i32(self.age, "age") {
            errors.push(e);
        }
        let age_range = InRange { min: 18, max: 120 };
        if let Err(e) = age_range.validate(self.age, "age") {
            errors.push(e);
        }

        // Validate password
        if let Err(e) = MinLength(8).validate(&self.password, "password") {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UpdateUserDto {
    pub name: Option<String>,
    pub email: Option<String>,
}

impl Validate for UpdateUserDto {
    fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate name if provided
        if let Some(ref name) = self.name {
            if let Err(e) = MinLength(2).validate(name, "name") {
                errors.push(e);
            }
        }

        // Validate email if provided
        if let Some(ref email) = self.email {
            if let Err(e) = IsEmail::validate(email, "email") {
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub age: i32,
}

// ========== Services ==========

#[injectable]
#[derive(Clone, Default)]
struct UserService;

impl UserService {
    fn create(&self, dto: CreateUserDto) -> Result<User, Error> {
        // In real app, save to database
        Ok(User {
            id: 1,
            name: dto.name,
            email: dto.email,
            age: dto.age,
        })
    }

    fn update(&self, id: i32, dto: UpdateUserDto) -> Result<User, Error> {
        // In real app, update in database
        Ok(User {
            id,
            name: dto.name.unwrap_or_else(|| "Updated".to_string()),
            email: dto
                .email
                .unwrap_or_else(|| "updated@example.com".to_string()),
            age: 30,
        })
    }
}

// ========== Controllers ==========

#[controller("/users")]
#[derive(Clone, Default)]
struct UserController {
    user_service: UserService,
}

impl UserController {
    fn create(&self, dto: CreateUserDto) -> Result<Json<User>, Error> {
        // Validation is automatic via ValidationPipe
        let user = self.user_service.create(dto)?;
        Ok(Json(user))
    }

    fn update(&self, id: i32, dto: UpdateUserDto) -> Result<Json<User>, Error> {
        let user = self.user_service.update(id, dto)?;
        Ok(Json(user))
    }
}

// ========== Module ==========

#[module(
    providers: [UserService],
    controllers: [UserController]
)]
#[derive(Default)]
struct AppModule;

#[tokio::main]
async fn main() {
    println!("✅ Armature Validation Example");
    println!("==============================\n");

    let app = create_app();

    println!("Server running on http://localhost:3018");
    println!();
    println!("Validation Features:");
    println!("  ✓ String validators (NotEmpty, MinLength, MaxLength)");
    println!("  ✓ Format validators (IsEmail, IsUrl, IsUuid)");
    println!("  ✓ Pattern validators (IsAlpha, IsAlphanumeric, IsNumeric)");
    println!("  ✓ Number validators (Min, Max, IsPositive, InRange)");
    println!("  ✓ Custom validators (Matches)");
    println!("  ✓ Validation pipe for automatic validation");
    println!("  ✓ Detailed error messages");
    println!();
    println!("Endpoints:");
    println!("  POST /users - Create user (with validation)");
    println!("  PUT /users/:id - Update user (with validation)");
    println!();
    println!("Example requests:");
    println!();
    println!("Valid:");
    println!("  curl -X POST http://localhost:3018/users \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!(
        "    -d '{{\"name\":\"John\",\"email\":\"john@example.com\",\"age\":30,\"password\":\"secure123\"}}'"
    );
    println!();
    println!("Invalid (will return validation errors):");
    println!("  curl -X POST http://localhost:3018/users \\");
    println!("    -H 'Content-Type: application/json' \\");
    println!("    -d '{{\"name\":\"\",\"email\":\"invalid\",\"age\":10,\"password\":\"short\"}}'");
    println!();

    if let Err(e) = app.listen(3018).await {
        eprintln!("Server error: {}", e);
    }
}

fn create_app() -> Application {
    let container = Container::new();
    let mut router = Router::new();

    // Register services
    let user_service = UserService::default();
    container.register(user_service.clone());

    let controller = UserController {
        user_service: user_service.clone(),
    };

    // POST /users - Create user with validation
    let create_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::POST,
        path: "/users".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = create_ctrl.clone();
            Box::pin(async move {
                // Use ValidationPipe to parse and validate
                let dto: CreateUserDto = ValidationPipe::parse(&req)?;
                ctrl.create(dto)?.into_response()
            })
        }),
    });

    // PUT /users/:id - Update user with validation
    let update_ctrl = controller.clone();
    router.add_route(Route {
        method: HttpMethod::PUT,
        path: "/users/:id".to_string(),
        handler: Arc::new(move |req| {
            let ctrl = update_ctrl.clone();
            Box::pin(async move {
                let id = req
                    .path_params
                    .get("id")
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);

                // Use ValidationPipe to parse and validate
                let dto: UpdateUserDto = ValidationPipe::parse(&req)?;
                ctrl.update(id, dto)?.into_response()
            })
        }),
    });

    Application::new(container, router)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_user() {
        let dto = CreateUserDto {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            password: "securepass123".to_string(),
        };

        assert!(dto.validate().is_ok());
    }

    #[test]
    fn test_invalid_email() {
        let dto = CreateUserDto {
            name: "John".to_string(),
            email: "invalid-email".to_string(),
            age: 30,
            password: "securepass123".to_string(),
        };

        let result = dto.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "email"));
    }

    #[test]
    fn test_age_validation() {
        let dto = CreateUserDto {
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 10, // Too young
            password: "securepass123".to_string(),
        };

        let result = dto.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "age"));
    }

    #[test]
    fn test_short_password() {
        let dto = CreateUserDto {
            name: "John".to_string(),
            email: "john@example.com".to_string(),
            age: 30,
            password: "short".to_string(), // Too short
        };

        let result = dto.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.field == "password"));
    }

    #[test]
    fn test_multiple_errors() {
        let dto = CreateUserDto {
            name: "".to_string(),         // Empty
            email: "invalid".to_string(), // Invalid format
            age: -5,                      // Negative
            password: "x".to_string(),    // Too short
        };

        let result = dto.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.len() >= 4); // Multiple validation errors
    }
}
