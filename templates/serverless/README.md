# Armature Serverless Deployment Templates

Templates for deploying Armature applications to serverless and container platforms.

## Quick Links

| Platform | Directory | Dockerfile |
|----------|-----------|------------|
| AWS Lambda | [`lambda/`](../lambda/) | `Dockerfile`, `Dockerfile.arm64` |
| Google Cloud Run | [`cloudrun/`](../cloudrun/) | `Dockerfile`, `Dockerfile.cloudbuild` |
| Azure Container Apps | [`azure-container/`](../azure-container/) | `Dockerfile` |
| Azure Functions | [`azure-container/`](../azure-container/) | `Dockerfile.functions` |

---

## AWS Lambda

Deploy Armature applications as Lambda functions using container images.

### Quick Start

```bash
# Build container
docker build -t my-lambda -f templates/lambda/Dockerfile .

# Tag for ECR
docker tag my-lambda:latest 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest

# Push
aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com
docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest

# Deploy
aws lambda create-function \
  --function-name my-function \
  --package-type Image \
  --code ImageUri=123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest \
  --role arn:aws:iam::123456789:role/lambda-role
```

### ARM64 (Graviton2) Support

For better price/performance, use ARM64:

```bash
docker buildx build --platform linux/arm64 -t my-lambda-arm64 -f templates/lambda/Dockerfile.arm64 .

aws lambda create-function \
  --function-name my-function \
  --package-type Image \
  --architectures arm64 \
  --code ImageUri=123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda-arm64:latest \
  --role arn:aws:iam::123456789:role/lambda-role
```

### Using cargo-lambda (Recommended)

```bash
# Install cargo-lambda
cargo install cargo-lambda

# Build
cargo lambda build --release

# Deploy directly
cargo lambda deploy --iam-role arn:aws:iam::123456789:role/lambda-role
```

---

## Google Cloud Run

Deploy Armature applications to Cloud Run with automatic scaling.

### Quick Start

```bash
# Build and push
gcloud builds submit --tag gcr.io/PROJECT_ID/armature-app

# Deploy
gcloud run deploy armature-app \
  --image gcr.io/PROJECT_ID/armature-app \
  --platform managed \
  --region us-central1 \
  --allow-unauthenticated
```

### Using Cloud Build

```bash
# Submit build with cloudbuild.yaml
gcloud builds submit --config=templates/cloudrun/cloudbuild.yaml

# Or with substitutions
gcloud builds submit \
  --config=templates/cloudrun/cloudbuild.yaml \
  --substitutions=_SERVICE=my-service,_REGION=us-central1
```

### Recommended Settings

```bash
gcloud run deploy armature-app \
  --image gcr.io/PROJECT_ID/armature-app \
  --platform managed \
  --region us-central1 \
  --memory 256Mi \
  --cpu 1 \
  --min-instances 0 \
  --max-instances 10 \
  --timeout 300s \
  --concurrency 80 \
  --allow-unauthenticated
```

---

## Azure Container Apps

Deploy Armature applications to Azure Container Apps.

### Quick Start

```bash
# Build
docker build -t armature-app -f templates/azure-container/Dockerfile .

# Tag for ACR
docker tag armature-app myregistry.azurecr.io/armature-app:latest

# Push
az acr login --name myregistry
docker push myregistry.azurecr.io/armature-app:latest

# Deploy
az containerapp create \
  --name armature-app \
  --resource-group my-rg \
  --environment my-env \
  --image myregistry.azurecr.io/armature-app:latest \
  --target-port 80 \
  --ingress external \
  --cpu 0.5 \
  --memory 1.0Gi
```

### Using YAML Configuration

```bash
# Deploy with YAML
az containerapp create \
  --name armature-app \
  --resource-group my-rg \
  --yaml templates/azure-container/containerapp.yaml
```

---

## Azure Functions

Deploy Armature applications as Azure Functions custom handlers.

### Quick Start

```bash
# Build
docker build -t armature-functions -f templates/azure-container/Dockerfile.functions .

# Push
az acr login --name myregistry
docker push myregistry.azurecr.io/armature-functions:latest

# Create Function App
az functionapp create \
  --name my-function-app \
  --resource-group my-rg \
  --storage-account mystorageaccount \
  --plan my-plan \
  --deployment-container-image-name myregistry.azurecr.io/armature-functions:latest \
  --functions-version 4
```

### Required Configuration Files

**host.json:**
```json
{
  "version": "2.0",
  "customHandler": {
    "description": {
      "defaultExecutablePath": "handler"
    },
    "enableForwardingHttpRequest": true
  }
}
```

**api/function.json:**
```json
{
  "bindings": [
    {
      "type": "httpTrigger",
      "direction": "in",
      "name": "req",
      "methods": ["get", "post", "put", "delete"],
      "route": "{*route}"
    },
    {
      "type": "http",
      "direction": "out",
      "name": "$return"
    }
  ]
}
```

---

## Local Development

Each platform has a `docker-compose.yml` for local development:

```bash
# Lambda (with LocalStack)
cd templates/lambda
docker-compose up -d

# Cloud Run (with GCP emulators)
cd templates/cloudrun
docker-compose up -d

# Azure (with Azurite)
cd templates/azure-container
docker-compose up -d
```

---

## Image Size Comparison

| Platform | Base Image | Approx. Size |
|----------|------------|--------------|
| Lambda | `public.ecr.aws/lambda/provided:al2023` | ~50-80MB |
| Cloud Run | `gcr.io/distroless/static` | ~15-25MB |
| Azure Container Apps | `alpine:3.20` | ~20-30MB |
| Azure Functions | `mcr.microsoft.com/azure-functions/base` | ~200-300MB |

---

## Best Practices

### 1. Use Multi-Stage Builds
All Dockerfiles use multi-stage builds to minimize image size.

### 2. Static Linking with musl
Rust binaries are built with `x86_64-unknown-linux-musl` for fully static linking.

### 3. Non-Root Users
All containers run as non-root users for security.

### 4. Health Checks
All containers include health check endpoints:
- `/health` - Overall health
- `/health/live` - Liveness probe
- `/health/ready` - Readiness probe

### 5. Environment Variables
Configure via environment variables:
- `PORT` - HTTP port (platform-specific default)
- `RUST_LOG` - Log level
- `DATABASE_URL` - Database connection
- `REDIS_URL` - Redis connection

---

## Troubleshooting

### Lambda Cold Start
- Use ARM64 for better cold start times
- Keep binaries small (<50MB recommended)
- Use provisioned concurrency for critical paths

### Cloud Run Cold Start
- Set `min-instances` >= 1 for critical services
- Use startup probes with appropriate timeouts
- Keep container image small

### Azure Cold Start
- Use Premium plan for lower latency
- Configure startup probes appropriately
- Consider always-on for critical services

