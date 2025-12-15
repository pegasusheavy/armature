# AWS Lambda Deployment Templates

Deploy Armature applications to AWS Lambda using container images.

## Files

| File | Description |
|------|-------------|
| `Dockerfile` | x86_64 Lambda container image |
| `Dockerfile.arm64` | ARM64 (Graviton2) Lambda container image |
| `docker-compose.yml` | Local development with LocalStack |

## Quick Start

### Using cargo-lambda (Recommended)

```bash
# Install
cargo install cargo-lambda

# Build
cargo lambda build --release

# Deploy
cargo lambda deploy my-function --iam-role arn:aws:iam::123456789:role/lambda-role
```

### Using Docker

```bash
# Build
docker build -t my-lambda -f Dockerfile .

# Tag for ECR
docker tag my-lambda:latest 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest

# Login to ECR
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin 123456789.dkr.ecr.us-east-1.amazonaws.com

# Push
docker push 123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest

# Create Lambda
aws lambda create-function \
  --function-name my-function \
  --package-type Image \
  --code ImageUri=123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda:latest \
  --role arn:aws:iam::123456789:role/lambda-role
```

## ARM64 (Graviton2)

ARM64 offers up to 34% better price/performance:

```bash
# Build for ARM64
docker buildx build --platform linux/arm64 -t my-lambda-arm64 -f Dockerfile.arm64 .

# Deploy with ARM64 architecture
aws lambda create-function \
  --function-name my-function \
  --package-type Image \
  --architectures arm64 \
  --code ImageUri=123456789.dkr.ecr.us-east-1.amazonaws.com/my-lambda-arm64:latest \
  --role arn:aws:iam::123456789:role/lambda-role
```

## Local Development

```bash
# Start LocalStack
docker-compose up -d

# Test locally
curl http://localhost:9000/
```

## Armature Integration

```rust
use armature_lambda::{LambdaRuntime, init_tracing};

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    init_tracing();

    let app = Application::create::<AppModule>();
    LambdaRuntime::new(app).run().await
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` |
| `AWS_REGION` | AWS region | from Lambda |
| `AWS_LAMBDA_FUNCTION_NAME` | Function name | from Lambda |

