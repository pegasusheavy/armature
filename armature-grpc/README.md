# armature-grpc

gRPC server and client support for the Armature framework.

## Features

- **Tonic Integration** - Built on the Tonic gRPC library
- **Code Generation** - Protobuf compilation support
- **Streaming** - Unary, server, client, and bidirectional streaming
- **Interceptors** - Request/response middleware
- **TLS** - Secure connections with rustls

## Installation

```toml
[dependencies]
armature-grpc = "0.1"
```

## Quick Start

### Server

```rust
use armature_grpc::{GrpcServer, Request, Response, Status};

pub struct MyService;

#[tonic::async_trait]
impl Greeter for MyService {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() {
    GrpcServer::builder()
        .add_service(GreeterServer::new(MyService))
        .serve("0.0.0.0:50051")
        .await
        .unwrap();
}
```

### Client

```rust
use armature_grpc::GrpcClient;

let client = GreeterClient::connect("http://localhost:50051").await?;
let response = client.say_hello(HelloRequest { name: "World".into() }).await?;
println!("Response: {}", response.into_inner().message);
```

## License

MIT OR Apache-2.0

