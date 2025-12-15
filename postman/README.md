# Armature Postman Collection

This directory contains Postman collections for testing Armature framework examples.

## Files

- `Armature_Examples.postman_collection.json` - Complete collection with 10 folders covering all major examples
- `Armature_Examples.postman_environment.json` - Environment variables for local development

## Quick Start

### 1. Import into Postman

1. Open Postman
2. Click **Import** (top-left)
3. Drag and drop both JSON files, or select them manually
4. The collection and environment will be imported

### 2. Select the Environment

1. Click the environment dropdown (top-right, next to the eye icon)
2. Select **"Armature Examples - Local"**

### 3. Run an Example

First, start the relevant Armature example server:

```bash
cd /path/to/armature

# Run a specific example
cargo run --example simple --features full
```

Then open the corresponding folder in Postman and test the endpoints.

## Collection Overview

| Folder | Example | Port | Description |
|--------|---------|------|-------------|
| 01. Simple API | `simple` | 3001 | Basic routing and JSON responses |
| 02. REST API | `rest_api` | 3002 | CRUD operations for tasks |
| 03. CRUD API | `crud_api` | 3000 | Complete user CRUD with pagination |
| 04. Auth API | `auth_api` | 3000 | JWT authentication |
| 05. Real-time API | `realtime_api` | 3000 | Chat, SSE, and WebSocket info |
| 06. GraphQL API | `graphql_api` | 3007 | GraphQL queries and mutations |
| 07. Pagination | `pagination_example` | 3000 | Advanced filtering and sorting |
| 08. Validation | `validation_example` | 3018 | Input validation examples |
| 09. Metrics | `metrics_example` | 3000 | Prometheus metrics |
| 10. Health Checks | (various) | 3000 | Common health endpoints |

## Example Commands

### Simple API (Port 3001)

```bash
# Start server
cargo run --example simple --features full

# Test with curl
curl http://localhost:3001/api/hello
curl -X POST http://localhost:3001/api/echo \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello!"}'
```

### REST API (Port 3002)

```bash
# Start server
cargo run --example rest_api --features full

# Test with curl
curl http://localhost:3002/tasks
curl http://localhost:3002/tasks/1
curl -X POST http://localhost:3002/tasks \
  -H "Content-Type: application/json" \
  -d '{"id": 0, "title": "New Task", "completed": false}'
```

### GraphQL API (Port 3007)

```bash
# Start server
cargo run --example graphql_api --features full

# Test with curl
curl -X POST http://localhost:3007/graphql \
  -H "Content-Type: application/json" \
  -d '{"query": "{ books { id title author } }"}'
```

### Pagination Example (Port 3000)

```bash
# Start server
cargo run --example pagination_example --features full

# Test pagination
curl "http://localhost:3000/users?page=1&per_page=3"

# Test sorting
curl "http://localhost:3000/users?sort=-age,name"

# Test filtering
curl "http://localhost:3000/users?status=active&age__gte=25"

# Test field selection
curl "http://localhost:3000/users?fields=id,name,email"

# Combined
curl "http://localhost:3000/users?page=1&sort=-age&status=active&fields=id,name"
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `base_url` | Base URL for all requests | `http://localhost` |
| `jwt_token` | JWT token for authenticated requests | (empty) |
| `api_key` | API key for rate limit bypass | (empty) |

### Setting JWT Token

After logging in with the Auth API:

1. Copy the `token` from the response
2. In Postman, click the eye icon next to the environment dropdown
3. Set `jwt_token` to the copied value
4. Now protected endpoints will work

## Tips

### Running Multiple Examples

Some examples use the same port (3000). Only run one at a time, or modify the example to use a different port.

### Postman Runner

Use the Collection Runner to test multiple requests:

1. Click **Runner** (bottom of the sidebar)
2. Select a folder or the entire collection
3. Click **Run** to execute all requests sequentially

### Variables in Requests

All requests use `{{base_url}}` which defaults to `http://localhost`. To test against a remote server:

1. Edit the environment
2. Change `base_url` to your server's address

### Authentication Workflow

For the Auth API:

1. Run **Register User** to create an account
2. Run **Login** to get a JWT token
3. Copy the token and set it in the environment
4. Run **Get Current User** (uses `{{jwt_token}}`)

## Contributing

To add new requests:

1. Run the example and identify endpoints from console output
2. Create requests in Postman
3. Export the collection
4. Update this README with usage instructions

## Troubleshooting

### "Connection refused"

The example server isn't running. Start it with:

```bash
cargo run --example <example_name> --features full
```

### "Port already in use"

Another example or process is using the port. Either:
- Stop the other process
- Or modify the example code to use a different port

### "Invalid JSON"

Check your request body syntax. Postman highlights JSON errors in red.

### "Validation errors"

The validation example intentionally shows error cases. Check the request description for expected behavior.

