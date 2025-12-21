# armature-messaging

Message broker integrations for the Armature framework.

## Features

- **RabbitMQ** - AMQP messaging
- **Kafka** - Event streaming
- **NATS** - Cloud-native messaging
- **AWS SQS/SNS** - AWS messaging services
- **Unified API** - Consistent interface across brokers

## Installation

```toml
[dependencies]
armature-messaging = "0.1"
```

## Quick Start

### RabbitMQ

```rust
use armature_messaging::rabbitmq::RabbitMQ;

let mq = RabbitMQ::connect("amqp://localhost:5672").await?;

// Publish
mq.publish("queue", message).await?;

// Subscribe
mq.subscribe("queue", |msg| async move {
    println!("Received: {:?}", msg);
    Ok(())
}).await?;
```

### Kafka

```rust
use armature_messaging::kafka::Kafka;

let kafka = Kafka::connect("localhost:9092").await?;

// Produce
kafka.produce("topic", key, value).await?;

// Consume
kafka.consume("topic", "group", |msg| async move {
    println!("Received: {:?}", msg);
    Ok(())
}).await?;
```

### NATS

```rust
use armature_messaging::nats::Nats;

let nats = Nats::connect("localhost:4222").await?;

// Publish
nats.publish("subject", message).await?;

// Subscribe
nats.subscribe("subject", |msg| async move {
    println!("Received: {:?}", msg);
    Ok(())
}).await?;
```

## License

MIT OR Apache-2.0

