# Armature Project Templates

Starter templates for building applications with the Armature framework.

## Available Templates

### Application Templates

| Template | Description | Features |
|----------|-------------|----------|
| [api-minimal](./api-minimal/) | Bare-bones REST API | Basic routing, JSON responses |
| [api-full](./api-full/) | Full-featured API | Auth, validation, OpenAPI, Docker |
| [graphql-api](./graphql-api/) | GraphQL API server | Queries, mutations, subscriptions, playground |
| [microservice](./microservice/) | Queue-connected microservice | Job worker, health checks, Docker |

### Serverless Deployment Templates

| Template | Platform | Features |
|----------|----------|----------|
| [lambda](./lambda/) | AWS Lambda | Container images, ARM64/Graviton2, LocalStack |
| [cloudrun](./cloudrun/) | Google Cloud Run | Distroless, Cloud Build, GCP emulators |
| [azure-container](./azure-container/) | Azure Container Apps | Container Apps, Azure Functions, Azurite |

### Infrastructure Templates

| Template | Description | Features |
|----------|-------------|----------|
| [k8s](./k8s/) | Kubernetes manifests | Deployment, Service, Ingress, HPA, PDB, NetworkPolicy |
| [helm](./helm/) | Helm charts | Parameterized K8s deployment |
| [jenkins](./jenkins/) | Jenkins pipelines | CI/CD templates |

## Quick Start

### Using a Template

1. Copy the template directory:
   ```bash
   cp -r templates/api-minimal my-project
   cd my-project
   ```

2. Update `Cargo.toml` with your project name:
   ```toml
   [package]
   name = "my-project"
   ```

3. Copy `.env.example` to `.env` and configure:
   ```bash
   cp .env.example .env
   ```

4. Run the project:
   ```bash
   cargo run
   ```

## Template Details

### api-minimal

The simplest starting point. Perfect for:
- Learning Armature
- Small APIs
- Prototyping

Features:
- Single-file implementation
- Basic health check endpoint
- JSON response helpers

### api-full

Production-ready API template. Perfect for:
- SaaS backends
- Public APIs
- Enterprise applications

Features:
- JWT authentication
- Request validation
- OpenAPI documentation
- Structured logging
- Docker support
- Health checks
- CORS configuration

### graphql-api

GraphQL API server. Perfect for:
- Modern frontend applications
- Mobile app backends
- APIs requiring flexible queries

Features:
- GraphQL Playground/GraphiQL
- Query, Mutation, Subscription support
- Type-safe schema with async-graphql
- Pagination patterns
- Authentication integration
- Health endpoints

### microservice

Background job processor. Perfect for:
- Email/notification services
- Data processing pipelines
- Async task handlers

Features:
- Redis job queue
- Retry with backoff
- Health endpoints
- Graceful shutdown
- Docker support

## Customization

All templates are designed to be customized. Common modifications:

### Adding Database Support

```toml
# Cargo.toml
[dependencies]
sqlx = { version = "0.7", features = ["runtime-tokio", "postgres"] }
```

### Adding Authentication

```toml
# Cargo.toml
[dependencies]
armature = { version = "0.1", features = ["auth"] }
```

### Adding Rate Limiting

```toml
# Cargo.toml
[dependencies]
armature = { version = "0.1", features = ["ratelimit"] }
```

---

## Serverless Deployment

### AWS Lambda

```bash
# Using cargo-lambda (recommended)
cargo lambda build --release
cargo lambda deploy

# Using Docker
docker build -t my-lambda -f templates/lambda/Dockerfile .
```

See [templates/lambda/](./lambda/) for details including ARM64 support.

### Google Cloud Run

```bash
# Quick deploy
gcloud builds submit --tag gcr.io/PROJECT/armature-app
gcloud run deploy armature-app --image gcr.io/PROJECT/armature-app --platform managed

# Using Cloud Build
gcloud builds submit --config=templates/cloudrun/cloudbuild.yaml
```

See [templates/cloudrun/](./cloudrun/) for Cloud Build configuration.

### Azure Container Apps

```bash
# Deploy
docker build -t myregistry.azurecr.io/app -f templates/azure-container/Dockerfile .
az acr login --name myregistry
docker push myregistry.azurecr.io/app
az containerapp create --name app --image myregistry.azurecr.io/app
```

See [templates/azure-container/](./azure-container/) for Azure Functions support.

---

## Infrastructure

### Kubernetes

```bash
# Deploy with kubectl
kubectl apply -k templates/k8s/

# Or with Helm
helm install armature templates/helm/armature/ -f values.yaml
```

### Local Development

Each serverless template includes a `docker-compose.yml` with emulators:

```bash
# Lambda with LocalStack
cd templates/lambda && docker-compose up -d

# Cloud Run with GCP emulators
cd templates/cloudrun && docker-compose up -d

# Azure with Azurite
cd templates/azure-container && docker-compose up -d
```

---

## Contributing

To add a new template:

1. Create a new directory under `templates/`
2. Include at minimum:
   - `Cargo.toml`
   - `src/main.rs`
   - `.env.example`
   - `README.md` (optional but recommended)
3. Update this README with the new template
4. Test the template works out of the box

## License

All templates are provided under the same license as Armature (Apache-2.0).

