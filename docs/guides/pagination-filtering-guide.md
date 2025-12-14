# Pagination & Filtering Guide

Comprehensive guide to pagination, sorting, filtering, search, and field selection in Armature.

## Table of Contents

- [Overview](#overview)
- [Pagination](#pagination)
- [Sorting](#sorting)
- [Filtering](#filtering)
- [Search](#search)
- [Field Selection](#field-selection)
- [Combined Queries](#combined-queries)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Summary](#summary)

---

## Overview

Armature provides powerful utilities for building flexible, performant APIs with:

- **Offset Pagination** - Traditional page-based pagination
- **Cursor Pagination** - For real-time/streaming data
- **Multi-field Sorting** - Sort by multiple fields with direction
- **Query Filtering** - Rich filter operators
- **Full-text Search** - Search integration points
- **Field Selection** - Sparse fieldsets (GraphQL-like)

All features parse from standard query parameters and work together seamlessly.

---

## Pagination

### Offset Pagination

Traditional page-based pagination using page number and page size.

#### Usage

```rust
use armature_core::*;
use std::collections::HashMap;

// Parse from query params
let mut params = HashMap::new();
params.insert("page".to_string(), "2".to_string());
params.insert("per_page".to_string(), "50".to_string());

let pagination = OffsetPagination::from_query_params(&params);

// Use in database query
let offset = pagination.offset(); // 50
let limit = pagination.limit();   // 50
```

#### Query Parameters

```
GET /users?page=2&per_page=50
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `page` | Page number (1-indexed) | 1 |
| `per_page` or `limit` | Items per page | 20 |

#### Calculate Offset

```rust
let pagination = OffsetPagination::new(3, 20);
let offset = pagination.offset(); // 40 (page 3, skip first 40)
```

#### Pagination Metadata

```rust
let total_items = 95;

pagination.total_pages(total_items); // 5
pagination.has_next(total_items);    // true (if page < 5)
pagination.has_prev();               // true (if page > 1)
```

#### Response Format

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
}

let users: Vec<User> = fetch_users(pagination.offset(), pagination.limit());
let total = count_users();

let response = PaginatedResponse::new(users, pagination, total);
```

Response JSON:

```json
{
  "data": [
    {"id": 21, "name": "Alice"},
    {"id": 22, "name": "Bob"}
  ],
  "meta": {
    "page": 2,
    "per_page": 20,
    "total": 95,
    "total_pages": 5,
    "has_next": true,
    "has_prev": true
  }
}
```

### Cursor Pagination

Opaque cursor-based pagination for real-time data and infinite scroll.

#### Usage

```rust
let pagination = CursorPagination::from_query_params(&params);

// cursor: Option<String>
// limit: usize
```

#### Query Parameters

```
GET /users?cursor=eyJpZCI6MTIzfQ&limit=20
```

| Parameter | Description | Default |
|-----------|-------------|---------|
| `cursor` | Opaque cursor for next page | None |
| `limit` | Items per page | 20 |

#### Response Format

```rust
let users: Vec<User> = fetch_users_after_cursor(cursor, limit);
let next_cursor = encode_cursor(users.last());

let response = PaginatedResponse::with_cursor(
    users,
    limit,
    total,
    next_cursor
);
```

Response JSON:

```json
{
  "data": [...],
  "meta": {
    "per_page": 20,
    "total": 1000,
    "has_next": true,
    "has_prev": false,
    "next_cursor": "eyJpZCI6MTIzfQ"
  }
}
```

#### When to Use Cursor vs Offset

| Offset Pagination | Cursor Pagination |
|-------------------|-------------------|
| ✅ Fixed datasets | ✅ Real-time data |
| ✅ Jump to page N | ✅ Infinite scroll |
| ✅ Show total pages | ✅ High-volume inserts |
| ❌ Inconsistent with frequent updates | ❌ Can't jump to arbitrary page |

---

## Sorting

Multi-field sorting with ascending/descending order.

### Usage

```rust
let params = HashMap::from([
    ("sort".to_string(), "-created_at,name".to_string())
]);

let sorting = SortParams::from_query(&params);
```

### Query Parameters

```
GET /users?sort=-created_at,+name,email
```

**Format:** Comma-separated field names with optional prefix:
- `-field` → Descending (DESC)
- `+field` or `field` → Ascending (ASC)

### Examples

| Query | Meaning |
|-------|---------|
| `?sort=name` | Sort by name ascending |
| `?sort=-created_at` | Sort by created_at descending |
| `?sort=-age,name` | Sort by age DESC, then name ASC |
| `?sort=+email,-created_at` | Sort by email ASC, then created_at DESC |

### In Code

```rust
let sorting = SortParams::from_query(&params);

// Check if sorting is specified
if !sorting.is_empty() {
    // Apply sorting to database query
    for field in &sorting.fields {
        query.order_by(&field.field, field.direction);
    }
}

// Or generate SQL
if let Some(sql) = sorting.to_sql() {
    // "created_at DESC, name ASC"
    query = format!("{} ORDER BY {}", query, sql);
}
```

### SortField API

```rust
use armature_core::*;

// Create sort fields
let sort = SortField::asc("name");
let sort = SortField::desc("created_at");
let sort = SortField::new("email", SortDirection::Asc);

// Parse from string
let sort = SortField::from_str("-created_at");
assert_eq!(sort.field, "created_at");
assert_eq!(sort.direction, SortDirection::Desc);

// Convert to SQL
let sql = sort.to_sql(); // "created_at DESC"
```

---

## Filtering

Rich query parameter filtering with multiple operators.

### Usage

```rust
let params = HashMap::from([
    ("status".to_string(), "active".to_string()),
    ("age__gte".to_string(), "18".to_string()),
    ("name__contains".to_string(), "john".to_string())
]);

let filters = FilterParams::from_query(&params);
```

### Query Parameters

**Format:** `field__operator=value`

```
GET /users?status=active&age__gte=18&name__contains=john
```

### Supported Operators

| Operator | Query Param | SQL | Example |
|----------|-------------|-----|---------|
| Equal | `field=value` | `=` | `status=active` |
| Not Equal | `field__ne=value` | `!=` | `status__ne=inactive` |
| Greater Than | `field__gt=value` | `>` | `age__gt=18` |
| Greater or Equal | `field__gte=value` | `>=` | `age__gte=18` |
| Less Than | `field__lt=value` | `<` | `age__lt=65` |
| Less or Equal | `field__lte=value` | `<=` | `age__lte=65` |
| In List | `field__in=val1,val2` | `IN` | `status__in=active,pending` |
| Not In | `field__not_in=val1,val2` | `NOT IN` | `role__not_in=admin` |
| Contains | `field__contains=value` | `LIKE %value%` | `name__contains=john` |
| Starts With | `field__starts_with=value` | `LIKE value%` | `email__starts_with=admin` |
| Ends With | `field__ends_with=value` | `LIKE %value` | `domain__ends_with=.com` |
| Is Null | `field__is_null=true` | `IS NULL` | `deleted_at__is_null=true` |
| Is Not Null | `field__is_not_null=true` | `IS NOT NULL` | `email__is_not_null=true` |

### Examples

```
# Simple equality
?status=active

# Greater than or equal
?age__gte=18

# Contains substring
?name__contains=john

# Multiple filters (AND)
?status=active&age__gte=18&role__ne=admin

# In list
?status__in=active,pending,approved
```

### In Code

```rust
let filters = FilterParams::from_query(&params);

// Check if filters are specified
if !filters.is_empty() {
    // Apply filters
    for condition in &filters.conditions {
        match condition.operator {
            FilterOperator::Eq => {
                query.where_eq(&condition.field, &condition.value);
            }
            FilterOperator::Gte => {
                query.where_gte(&condition.field, &condition.value);
            }
            // ... handle other operators
        }
    }
}

// Get specific filter
if let Some(status_filter) = filters.get("status") {
    // Use status filter
}
```

### FilterCondition API

```rust
use armature_core::*;

let condition = FilterCondition::new(
    "age",
    FilterOperator::Gte,
    Some("18".to_string())
);

// Access fields
println!("Field: {}", condition.field);     // "age"
println!("Operator: {:?}", condition.operator); // Gte
println!("Value: {:?}", condition.value);    // Some("18")

// Convert to SQL
let sql_op = condition.operator.to_sql(); // ">="
```

---

## Search

Full-text search integration.

### Usage

```rust
let params = HashMap::from([
    ("q".to_string(), "search term".to_string())
]);

let search = SearchParams::from_query(&params);

if search.is_active() {
    let query = search.query.unwrap();
    // Perform full-text search
}
```

### Query Parameters

```
GET /users?q=search+term
GET /users?search=john+doe
GET /users?q=keyword&search_fields=name,email,bio
```

| Parameter | Description |
|-----------|-------------|
| `q` or `search` | Search query |
| `search_fields` | Comma-separated fields to search in |

### Examples

```
# Basic search
?q=john

# Search in specific fields
?q=john&search_fields=name,email

# Combined with filters
?q=developer&status=active&age__gte=25
```

### Integration with Search Engines

```rust
let search = SearchParams::from_query(&params);

if search.is_active() {
    let query = search.query.unwrap();

    // Elasticsearch
    let results = elasticsearch_client
        .search(&query, &search.fields)
        .await?;

    // PostgreSQL full-text search
    let results = sqlx::query!(
        "SELECT * FROM users WHERE to_tsvector('english', name || ' ' || email) @@ plainto_tsquery($1)",
        query
    ).fetch_all(&pool).await?;

    // Meilisearch
    let results = meilisearch_client
        .index("users")
        .search()
        .with_query(&query)
        .execute()
        .await?;
}
```

---

## Field Selection

Sparse fieldsets allow clients to request only specific fields (like GraphQL).

### Usage

```rust
let params = HashMap::from([
    ("fields".to_string(), "id,name,email".to_string())
]);

let fields = FieldSelection::from_query(&params);

if fields.should_include("name") {
    // Include name field
}
```

### Query Parameters

```
GET /users?fields=id,name,email
GET /users?exclude=password,secret
```

| Parameter | Description |
|-----------|-------------|
| `fields` | Comma-separated fields to include |
| `exclude` | Comma-separated fields to exclude |

### Examples

```
# Include only specific fields
?fields=id,name,email

# Exclude sensitive fields
?exclude=password,secret_key,ssn

# Cannot combine include and exclude
# (include takes precedence if both specified)
```

### In Code

```rust
let fields = FieldSelection::from_query(&params);

if fields.is_active() {
    // Filter response fields
    let filtered: Vec<serde_json::Value> = users
        .iter()
        .map(|user| {
            let mut obj = serde_json::json!({});

            if fields.should_include("id") {
                obj["id"] = json!(user.id);
            }
            if fields.should_include("name") {
                obj["name"] = json!(user.name);
            }
            if fields.should_include("email") {
                obj["email"] = json!(user.email);
            }
            // Don't include password

            obj
        })
        .collect();

    return Ok(HttpResponse::ok().with_json(&filtered)?);
}
```

### Database Optimization

```rust
// Select only requested fields from database
let mut select = vec![];

if fields.should_include("id") {
    select.push("id");
}
if fields.should_include("name") {
    select.push("name");
}

let query = format!("SELECT {} FROM users", select.join(", "));
```

---

## Combined Queries

All query features work together seamlessly.

### Example: Complete Query

```
GET /users?page=2&per_page=20&sort=-created_at,name&status=active&age__gte=25&q=developer&fields=id,name,email
```

Breakdown:
- **Pagination:** Page 2, 20 items per page
- **Sorting:** By created_at DESC, then name ASC
- **Filtering:** Active users aged 25+
- **Search:** Contains "developer"
- **Fields:** Return only id, name, email

### Parsing All Parameters

```rust
use armature_core::*;

let query = QueryParams::from_hashmap(&req.query_params);

// Access all parsed parameters
let pagination = query.pagination; // OffsetPagination
let sorting = query.sort;          // SortParams
let filters = query.filter;        // FilterParams
let search = query.search;         // SearchParams
let fields = query.fields;         // FieldSelection
```

### Complete Example

```rust
async fn list_users(req: HttpRequest) -> Result<HttpResponse, Error> {
    // Parse all query parameters
    let query = QueryParams::from_hashmap(&req.query_params);

    // Start building database query
    let mut db_query = String::from("SELECT * FROM users WHERE 1=1");

    // Apply filters
    for condition in &query.filter.conditions {
        db_query.push_str(&format!(
            " AND {} {} {}",
            condition.field,
            condition.operator.to_sql(),
            condition.value.as_ref().unwrap_or(&"NULL".to_string())
        ));
    }

    // Apply search
    if let Some(search_query) = &query.search.query {
        db_query.push_str(&format!(
            " AND (name LIKE '%{}%' OR email LIKE '%{}%')",
            search_query, search_query
        ));
    }

    // Apply sorting
    if let Some(order_by) = query.sort.to_sql() {
        db_query.push_str(&format!(" ORDER BY {}", order_by));
    }

    // Apply pagination
    db_query.push_str(&format!(
        " LIMIT {} OFFSET {}",
        query.pagination.limit(),
        query.pagination.offset()
    ));

    // Execute query
    let users: Vec<User> = sqlx::query_as(&db_query)
        .fetch_all(&pool)
        .await?;

    // Get total count (for pagination metadata)
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await?;

    // Create paginated response
    let response = PaginatedResponse::new(
        users,
        query.pagination,
        total as usize
    );

    Ok(HttpResponse::ok().with_json(&response)?)
}
```

---

## Best Practices

### 1. Set Maximum Page Size

```rust
pub const MAX_PAGE_SIZE: usize = 100;

let per_page = per_page.clamp(1, MAX_PAGE_SIZE);
```

Prevents clients from requesting too many items.

### 2. Provide Default Sorting

```rust
// Always have a default sort for consistent results
let sorting = SortParams::from_query(&params);

if sorting.is_empty() {
    sorting = SortParams::new(vec![SortField::desc("created_at")]);
}
```

### 3. Validate Filter Fields

```rust
const ALLOWED_FILTERS: &[&str] = &["status", "age", "role"];

for condition in &filters.conditions {
    if !ALLOWED_FILTERS.contains(&condition.field.as_str()) {
        return Err(Error::BadRequest(format!(
            "Filtering by '{}' is not allowed",
            condition.field
        )));
    }
}
```

### 4. Index Database Columns

```sql
-- Index filtered columns
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Composite index for common queries
CREATE INDEX idx_users_status_created ON users(status, created_at DESC);
```

### 5. Use Cursor Pagination for Real-time Data

For feeds with frequent inserts, cursor pagination prevents missing/duplicate items.

### 6. Document Available Filters

```rust
// Return available filters in API documentation
{
  "filters": {
    "status": ["active", "inactive", "pending"],
    "role": ["user", "admin", "moderator"]
  },
  "sortable_fields": ["created_at", "name", "age"],
  "searchable_fields": ["name", "email", "bio"]
}
```

### 7. Cache Total Counts

```rust
// Cache expensive COUNT queries
let cache_key = format!("users:count:{:?}", filters);

let total = if let Some(cached) = cache.get(&cache_key).await? {
    cached
} else {
    let count = count_users(&filters).await?;
    cache.set(&cache_key, count, Duration::from_secs(60)).await?;
    count
};
```

---

## Examples

### Example 1: Basic Pagination

```bash
curl "http://localhost:3000/users?page=2&per_page=20"
```

```rust
let pagination = OffsetPagination::from_query_params(&req.query_params);
let users = fetch_users(pagination.offset(), pagination.limit());
let response = PaginatedResponse::new(users, pagination, total);
```

### Example 2: Sorting

```bash
curl "http://localhost:3000/users?sort=-created_at,name"
```

```rust
let sorting = SortParams::from_query(&req.query_params);
// Sort by created_at DESC, then name ASC
```

### Example 3: Filtering

```bash
curl "http://localhost:3000/users?status=active&age__gte=18&role__ne=guest"
```

```rust
let filters = FilterParams::from_query(&req.query_params);
// WHERE status = 'active' AND age >= 18 AND role != 'guest'
```

### Example 4: Field Selection

```bash
curl "http://localhost:3000/users?fields=id,name,email"
```

```rust
let fields = FieldSelection::from_query(&req.query_params);
// Return only id, name, email (exclude password, etc.)
```

### Example 5: Combined Query

```bash
curl "http://localhost:3000/users?page=1&per_page=10&sort=-age&status=active&age__gte=25&fields=id,name"
```

```rust
let query = QueryParams::from_hashmap(&req.query_params);
// Page 1, 10 per page
// Sort by age DESC
// Filter: status=active AND age>=25
// Fields: id, name only
```

---

## Summary

**Key Points:**

1. **Offset Pagination** - Traditional page-based (page/per_page)
2. **Cursor Pagination** - For real-time data (cursor/limit)
3. **Multi-field Sorting** - `-field` for DESC, `+field` or `field` for ASC
4. **Rich Filtering** - `field__operator=value` format
5. **Search Integration** - `q` or `search` parameter
6. **Field Selection** - `fields` or `exclude` parameters
7. **Combined Queries** - All features work together

**Quick Reference:**

```rust
// Parse all parameters at once
let query = QueryParams::from_hashmap(&req.query_params);

// Or individually
let pagination = OffsetPagination::from_query_params(&params);
let sorting = SortParams::from_query(&params);
let filters = FilterParams::from_query(&params);
let search = SearchParams::from_query(&params);
let fields = FieldSelection::from_query(&params);
```

**Common Patterns:**

```
# Pagination
?page=2&per_page=50

# Sorting (- = DESC)
?sort=-created_at,name

# Filtering
?status=active&age__gte=18

# Search
?q=keyword

# Fields
?fields=id,name,email

# Combined
?page=1&sort=-age&status=active&fields=id,name
```

**Resources:**
- [Example: Pagination](../../examples/pagination_example.rs)
- [API: Pagination Module](../../armature-core/src/pagination.rs)

