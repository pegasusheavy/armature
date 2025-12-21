# armature-aws

AWS cloud services integration for the Armature framework.

## Features

- **S3** - Object storage
- **DynamoDB** - NoSQL database
- **SQS/SNS** - Message queues and topics
- **Secrets Manager** - Secure secrets
- **Parameter Store** - Configuration management
- **CloudWatch** - Logging and metrics

## Installation

```toml
[dependencies]
armature-aws = "0.1"
```

## Quick Start

```rust
use armature_aws::{S3Client, DynamoClient, SqsClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // S3
    let s3 = S3Client::new("us-east-1").await?;
    s3.put_object("bucket", "key", bytes).await?;

    // DynamoDB
    let dynamo = DynamoClient::new("us-east-1").await?;
    dynamo.put_item("table", item).await?;

    // SQS
    let sqs = SqsClient::new("us-east-1").await?;
    sqs.send_message("queue-url", "message").await?;

    Ok(())
}
```

## Credential Chain

Credentials are loaded from:
1. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
2. AWS credentials file (`~/.aws/credentials`)
3. IAM role (ECS, Lambda, EC2)

## License

MIT OR Apache-2.0

