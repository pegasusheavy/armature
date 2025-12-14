# Route Constraints Guide

Comprehensive guide to validating route parameters with Route Constraints in Armature.

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Built-in Constraints](#built-in-constraints)
- [Basic Usage](#basic-usage)
- [Custom Constraints](#custom-constraints)
- [Combining Constraints](#combining-constraints)
- [Best Practices](#best-practices)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Summary](#summary)

---

## Overview

Route Constraints validate path parameters at the routing level, before handlers are called. This provides:

- **Early validation** - Fail fast before business logic
- **Better error messages** - Clear validation errors
- **Type safety** - Ensure parameters match expected types
- **Clean handlers** - No validation code in handlers

---

## Features

- ✅ Built-in constraints (Int, UUID, Email, etc.)
- ✅ Custom constraint creation
- ✅ Composable constraints
- ✅ Clear error messages
- ✅ Type-safe validation
- ✅ No runtime overhead (validation only on match)

---

## Built-in Constraints

### IntConstraint

Validates that a parameter is a valid signed integer.

```rust
use armature_core::*;

let constraint = IntConstraint;

assert!(constraint.validate("123").is_ok());
assert!(constraint.validate("-456").is_ok());
assert!(constraint.validate("abc").is_err());
```

**Use cases:**
- User IDs
- Database primary keys
- Pagination offsets

### UIntConstraint

Validates that a parameter is a valid unsigned integer (≥ 0).

```rust
use armature_core::*;

let constraint = UIntConstraint;

assert!(constraint.validate("123").is_ok());
assert!(constraint.validate("0").is_ok());
assert!(constraint.validate("-1").is_err());
```

**Use cases:**
- Counts
- Page numbers
- Array indices

### FloatConstraint

Validates that a parameter is a valid floating-point number.

```rust
use armature_core::*;

let constraint = FloatConstraint;

assert!(constraint.validate("123.45").is_ok());
assert!(constraint.validate("-0.5").is_ok());
assert!(constraint.validate("abc").is_err());
```

**Use cases:**
- Prices
- Coordinates
- Percentages

### AlphaConstraint

Validates that a parameter contains only letters (a-z, A-Z).

```rust
use armature_core::*;

let constraint = AlphaConstraint;

assert!(constraint.validate("hello").is_ok());
assert!(constraint.validate("WORLD").is_ok());
assert!(constraint.validate("hello123").is_err());
```

**Use cases:**
- Names (first/last)
- Language codes
- Alphabetic identifiers

### AlphaNumConstraint

Validates that a parameter contains only letters and numbers.

```rust
use armature_core::*;

let constraint = AlphaNumConstraint;

assert!(constraint.validate("user123").is_ok());
assert!(constraint.validate("ABC").is_ok());
assert!(constraint.validate("user-123").is_err());
```

**Use cases:**
- Usernames
- Slugs
- Alphanumeric codes

### UuidConstraint

Validates that a parameter is a valid UUID (8-4-4-4-12 format).

```rust
use armature_core::*;

let constraint = UuidConstraint;

assert!(constraint.validate("550e8400-e29b-41d4-a716-446655440000").is_ok());
assert!(constraint.validate("not-a-uuid").is_err());
```

**Use cases:**
- Resource identifiers
- Session tokens
- Unique IDs

### EmailConstraint

Validates that a parameter is a valid email address.

```rust
use armature_core::*;

let constraint = EmailConstraint;

assert!(constraint.validate("user@example.com").is_ok());
assert!(constraint.validate("invalid-email").is_err());
```

**Use cases:**
- User lookup by email
- Email verification
- Contact forms

### LengthConstraint

Validates that a parameter has a specific length or length range.

```rust
use armature_core::*;

// Between 3 and 20 characters
let constraint = LengthConstraint::new(Some(3), Some(20));
assert!(constraint.validate("hello").is_ok());
assert!(constraint.validate("hi").is_err());

// At least 5 characters
let min_constraint = LengthConstraint::min(5);
assert!(min_constraint.validate("hello").is_ok());
assert!(min_constraint.validate("hi").is_err());

// At most 10 characters
let max_constraint = LengthConstraint::max(10);
assert!(max_constraint.validate("hello").is_ok());
assert!(max_constraint.validate("verylongstring").is_err());

// Exactly 5 characters
let exact_constraint = LengthConstraint::exact(5);
assert!(exact_constraint.validate("hello").is_ok());
assert!(exact_constraint.validate("world!").is_err());
```

**Use cases:**
- Username length validation
- Code length (postal codes, etc.)
- Fixed-length identifiers

### RangeConstraint

Validates that a number is within a specific range.

```rust
use armature_core::*;

// Between 1 and 100
let constraint = RangeConstraint::new(Some(1), Some(100));
assert!(constraint.validate("50").is_ok());
assert!(constraint.validate("0").is_err());

// At least 0
let min_constraint = RangeConstraint::min(0);

// At most 1000
let max_constraint = RangeConstraint::max(1000);
```

**Use cases:**
- Age validation
- Percentage validation (0-100)
- Page numbers

### EnumConstraint

Validates that a parameter is one of a set of allowed values.

```rust
use armature_core::*;

let constraint = EnumConstraint::new(vec![
    "active".to_string(),
    "inactive".to_string(),
    "pending".to_string(),
]);

assert!(constraint.validate("active").is_ok());
assert!(constraint.validate("unknown").is_err());
```

**Use cases:**
- Status values
- Sort directions (asc/desc)
- Enum-like parameters

### RegexConstraint

Validates that a parameter matches a regular expression pattern.

```rust
use armature_core::*;

// Only lowercase letters
let constraint = RegexConstraint::new(r"^[a-z]+$", "lowercase letters").unwrap();
assert!(constraint.validate("hello").is_ok());
assert!(constraint.validate("HELLO").is_err());
```

**Use cases:**
- Custom formats
- Complex patterns
- Domain-specific validation

---

## Basic Usage

### Single Constraint

```rust
use armature_core::*;

// Create route constraints
let constraints = RouteConstraints::new()
    .add("id", Box::new(IntConstraint));

// Validate parameters
let mut params = std::collections::HashMap::new();
params.insert("id".to_string(), "123".to_string());

assert!(constraints.validate(&params).is_ok());
```

### Multiple Constraints

```rust
use armature_core::*;

let constraints = RouteConstraints::new()
    .add("id", Box::new(UIntConstraint))
    .add("uuid", Box::new(UuidConstraint))
    .add("email", Box::new(EmailConstraint));

// All parameters must satisfy their constraints
```

### With Route

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("id", Box::new(IntConstraint))
    .add("name", Box::new(AlphaConstraint));

let route = Route {
    method: HttpMethod::GET,
    path: "/users/:id/:name".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            // Parameters are already validated!
            let id = req.path_params.get("id").unwrap();
            let name = req.path_params.get("name").unwrap();

            Ok(HttpResponse::ok())
        })
    }),
    constraints: Some(constraints),
};
```

---

## Custom Constraints

Implement the `RouteConstraint` trait:

```rust
use armature_core::*;

/// Custom constraint for US ZIP codes
struct ZipCodeConstraint;

impl RouteConstraint for ZipCodeConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        if value.len() == 5 && value.chars().all(|c| c.is_numeric()) {
            Ok(())
        } else {
            Err(format!("'{}' is not a valid ZIP code", value))
        }
    }

    fn description(&self) -> &str {
        "5-digit ZIP code"
    }
}

// Use it
let constraints = RouteConstraints::new()
    .add("zip", Box::new(ZipCodeConstraint));
```

### Custom Range Constraint

```rust
use armature_core::*;

struct AgeConstraint;

impl RouteConstraint for AgeConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        let age: u8 = value
            .parse()
            .map_err(|_| format!("'{}' is not a valid age", value))?;

        if age < 18 {
            Err("Must be at least 18 years old".to_string())
        } else if age > 120 {
            Err("Invalid age".to_string())
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        "age between 18 and 120"
    }
}
```

### Custom Format Constraint

```rust
use armature_core::*;

/// Phone number in format: (XXX) XXX-XXXX
struct PhoneConstraint;

impl RouteConstraint for PhoneConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        let regex = regex::Regex::new(r"^\(\d{3}\) \d{3}-\d{4}$").unwrap();

        if regex.is_match(value) {
            Ok(())
        } else {
            Err(format!("'{}' must be in format (XXX) XXX-XXXX", value))
        }
    }

    fn description(&self) -> &str {
        "US phone number"
    }
}
```

---

## Combining Constraints

### Multiple Parameters

```rust
use armature_core::*;

let constraints = RouteConstraints::new()
    .add("user_id", Box::new(UIntConstraint))
    .add("post_id", Box::new(UIntConstraint))
    .add("status", Box::new(EnumConstraint::new(vec![
        "active".to_string(),
        "archived".to_string(),
    ])));

// Route: /users/:user_id/posts/:post_id/:status
```

### Layered Validation

```rust
use armature_core::*;

// First validate it's a number, then validate range
let constraints = RouteConstraints::new()
    .add("age", Box::new(RangeConstraint::new(Some(18), Some(120))));

// RangeConstraint internally validates it's an integer first
```

---

## Best Practices

### 1. Validate Early

```rust
// ✅ GOOD - Validate at route level
let constraints = RouteConstraints::new()
    .add("id", Box::new(IntConstraint));

let route = Route {
    constraints: Some(constraints),
    ..route
};

// Handler is only called if validation passes
async fn handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id: i32 = req.path_params.get("id").unwrap().parse().unwrap();
    // Safe to unwrap - already validated
}
```

```rust
// ❌ BAD - Validate in handler
async fn handler(req: HttpRequest) -> Result<HttpResponse, Error> {
    let id = req.path_params.get("id")
        .ok_or_else(|| Error::BadRequest("Missing id".into()))?;

    let id: i32 = id.parse()
        .map_err(|_| Error::BadRequest("Invalid id".into()))?;

    // Lots of validation code in every handler
}
```

### 2. Use Appropriate Constraints

```rust
// ✅ GOOD - Specific constraints
let constraints = RouteConstraints::new()
    .add("id", Box::new(UIntConstraint))  // IDs are always positive
    .add("uuid", Box::new(UuidConstraint))  // UUIDs have specific format
    .add("email", Box::new(EmailConstraint));  // Emails have rules
```

```rust
// ❌ BAD - Too generic
let constraints = RouteConstraints::new()
    .add("id", Box::new(AlphaNumConstraint))  // Too broad
    .add("uuid", Box::new(AlphaNumConstraint));  // Accepts invalid UUIDs
```

### 3. Provide Clear Error Messages

```rust
// ✅ GOOD - Descriptive validation
impl RouteConstraint for CustomConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !is_valid(value) {
            Err(format!(
                "'{}' must be a valid product code (format: ABC-1234)",
                value
            ))
        } else {
            Ok(())
        }
    }
}
```

```rust
// ❌ BAD - Vague error
impl RouteConstraint for CustomConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !is_valid(value) {
            Err("Invalid".to_string())  // Not helpful!
        } else {
            Ok(())
        }
    }
}
```

### 4. Combine with Type-Safe Extractors

```rust
use armature_core::*;

// Constraint ensures it's valid
let constraints = RouteConstraints::new()
    .add("id", Box::new(UIntConstraint));

// Handler can safely extract
#[get("/users/:id")]
async fn get_user(
    #[param("id")] id: u64  // Guaranteed to be valid
) -> Result<HttpResponse, Error> {
    // No validation needed!
    Ok(HttpResponse::ok())
}
```

---

## API Reference

### RouteConstraint Trait

```rust
pub trait RouteConstraint: Send + Sync {
    fn validate(&self, value: &str) -> Result<(), String>;
    fn description(&self) -> &str;
}
```

### RouteConstraints

```rust
pub struct RouteConstraints

impl RouteConstraints {
    pub fn new() -> Self
    pub fn add(self, param: impl Into<String>, constraint: Box<dyn RouteConstraint>) -> Self
    pub fn add_mut(&mut self, param: impl Into<String>, constraint: Box<dyn RouteConstraint>)
    pub fn validate(&self, params: &HashMap<String, String>) -> Result<(), Error>
    pub fn is_empty(&self) -> bool
    pub fn len(&self) -> usize
}
```

### Built-in Constraints

| Constraint | Constructor | Description |
|------------|-------------|-------------|
| `IntConstraint` | Unit struct | Signed integer |
| `UIntConstraint` | Unit struct | Unsigned integer |
| `FloatConstraint` | Unit struct | Floating point |
| `AlphaConstraint` | Unit struct | Letters only |
| `AlphaNumConstraint` | Unit struct | Letters and numbers |
| `UuidConstraint` | Unit struct | UUID format |
| `EmailConstraint` | Unit struct | Email format |
| `LengthConstraint` | `new(min, max)` | String length |
| `RangeConstraint` | `new(min, max)` | Number range |
| `EnumConstraint` | `new(values)` | Enum values |
| `RegexConstraint` | `new(pattern, desc)` | Regex match |

---

## Examples

### Example 1: REST API with ID Validation

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("id", Box::new(UIntConstraint));

let route = Route {
    method: HttpMethod::GET,
    path: "/users/:id".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            let id = req.path_params.get("id").unwrap();
            // id is guaranteed to be a valid unsigned integer

            Ok(HttpResponse::ok()
                .with_json(&serde_json::json!({
                    "user_id": id
                }))?)
        })
    }),
    constraints: Some(constraints),
};

// Valid: GET /users/123
// Invalid: GET /users/abc  → 400 Bad Request
// Invalid: GET /users/-1   → 400 Bad Request
```

### Example 2: UUID-based Resources

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("uuid", Box::new(UuidConstraint));

let route = Route {
    method: HttpMethod::GET,
    path: "/resources/:uuid".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            let uuid = req.path_params.get("uuid").unwrap();
            // uuid is guaranteed to be valid UUID format

            Ok(HttpResponse::ok())
        })
    }),
    constraints: Some(constraints),
};

// Valid: GET /resources/550e8400-e29b-41d4-a716-446655440000
// Invalid: GET /resources/not-a-uuid  → 400 Bad Request
```

### Example 3: Status Filter

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("status", Box::new(EnumConstraint::new(vec![
        "active".to_string(),
        "inactive".to_string(),
        "pending".to_string(),
        "archived".to_string(),
    ])));

let route = Route {
    method: HttpMethod::GET,
    path: "/users/:status".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            let status = req.path_params.get("status").unwrap();
            // status is one of: active, inactive, pending, archived

            Ok(HttpResponse::ok())
        })
    }),
    constraints: Some(constraints),
};

// Valid: GET /users/active
// Valid: GET /users/pending
// Invalid: GET /users/unknown  → 400 Bad Request
```

### Example 4: Pagination with Range

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("page", Box::new(RangeConstraint::min(1)))
    .add("limit", Box::new(RangeConstraint::new(Some(1), Some(100))));

let route = Route {
    method: HttpMethod::GET,
    path: "/posts/:page/:limit".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            let page: u32 = req.path_params.get("page").unwrap().parse().unwrap();
            let limit: u32 = req.path_params.get("limit").unwrap().parse().unwrap();

            // page >= 1, limit between 1 and 100

            Ok(HttpResponse::ok())
        })
    }),
    constraints: Some(constraints),
};

// Valid: GET /posts/1/10
// Valid: GET /posts/5/100
// Invalid: GET /posts/0/10    → 400 Bad Request (page < 1)
// Invalid: GET /posts/1/101   → 400 Bad Request (limit > 100)
```

### Example 5: Username Validation

```rust
use armature_core::*;
use std::sync::Arc;

let constraints = RouteConstraints::new()
    .add("username", Box::new(AlphaNumConstraint));

let route = Route {
    method: HttpMethod::GET,
    path: "/users/:username".to_string(),
    handler: Arc::new(|req| {
        Box::pin(async move {
            let username = req.path_params.get("username").unwrap();
            // username contains only letters and numbers

            Ok(HttpResponse::ok())
        })
    }),
    constraints: Some(constraints),
};

// Valid: GET /users/john123
// Valid: GET /users/Alice
// Invalid: GET /users/john-doe  → 400 Bad Request
// Invalid: GET /users/john@doe  → 400 Bad Request
```

---

## Troubleshooting

### Issue: Constraint Not Applied

**Symptom:** Invalid parameters reach handler

**Cause:** Forgot to add constraints to route

**Solution:**
```rust
let route = Route {
    constraints: Some(constraints),  // Don't forget!
    ..route
};
```

### Issue: Too Strict Validation

**Symptom:** Valid requests are rejected

**Cause:** Constraint is too restrictive

**Solution:** Use a more appropriate constraint or create a custom one

```rust
// Too strict
let constraint = LengthConstraint::exact(5);

// Better
let constraint = LengthConstraint::new(Some(3), Some(20));
```

### Issue: Unclear Error Messages

**Symptom:** Users don't understand why request failed

**Cause:** Generic constraint error messages

**Solution:** Create custom constraint with specific messages

```rust
struct ProductCodeConstraint;

impl RouteConstraint for ProductCodeConstraint {
    fn validate(&self, value: &str) -> Result<(), String> {
        if !matches_format(value) {
            Err(format!(
                "'{}' must be a product code in format: CATEGORY-NUMBER (e.g., ELEC-1234)",
                value
            ))
        } else {
            Ok(())
        }
    }

    fn description(&self) -> &str {
        "product code (CATEGORY-NUMBER)"
    }
}
```

---

## Summary

**Key Points:**

1. **Validate early** with route constraints
2. **Use built-in constraints** for common cases
3. **Create custom constraints** for domain-specific validation
4. **Provide clear error messages** for users
5. **Combine with extractors** for type safety
6. **Fail fast** before handler execution

**Quick Reference:**

```rust
// Create constraints
let constraints = RouteConstraints::new()
    .add("id", Box::new(UIntConstraint))
    .add("uuid", Box::new(UuidConstraint))
    .add("status", Box::new(EnumConstraint::new(vec![
        "active".to_string(),
        "inactive".to_string(),
    ])));

// Add to route
let route = Route {
    method: HttpMethod::GET,
    path: "/users/:id/:uuid/:status".to_string(),
    handler: my_handler,
    constraints: Some(constraints),
};
```

**Benefits:**

- 🚀 **Early validation** - Fail fast
- 🛡️ **Type safety** - Ensure valid parameters
- 📝 **Clear errors** - Better error messages
- 🧹 **Clean handlers** - No validation code
- 🔧 **Maintainable** - Centralized validation
- 🎯 **Focused** - Handlers focus on business logic

