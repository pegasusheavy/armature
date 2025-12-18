//! Service Discovery Example
//!
//! Demonstrates service registration and discovery with load balancing.

use armature_discovery::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Service Discovery Example ===\n");

    // Using in-memory discovery for this example
    // In production, use ConsulDiscovery or EtcdDiscovery
    let discovery = InMemoryDiscovery::new();

    // 1. Register services
    println!("1. Registering Services:");

    let api1 = ServiceInstance::new("api-1", "api", "192.168.1.10", 8080)
        .with_tag("v1.0.0")
        .with_tag("production")
        .with_metadata("region", "us-east")
        .with_metadata("datacenter", "dc1")
        .with_health_check("http://192.168.1.10:8080/health");

    let api2 = ServiceInstance::new("api-2", "api", "192.168.1.11", 8080)
        .with_tag("v1.0.0")
        .with_tag("production")
        .with_metadata("region", "us-east")
        .with_metadata("datacenter", "dc1")
        .with_health_check("http://192.168.1.11:8080/health");

    let api3 = ServiceInstance::new("api-3", "api", "192.168.1.12", 8080)
        .with_tag("v1.0.0")
        .with_tag("production")
        .with_metadata("region", "us-west")
        .with_metadata("datacenter", "dc2")
        .with_health_check("http://192.168.1.12:8080/health");

    discovery.register(&api1).await?;
    discovery.register(&api2).await?;
    discovery.register(&api3).await?;

    println!("   ✅ Registered api-1 at {}", api1.url());
    println!("   ✅ Registered api-2 at {}", api2.url());
    println!("   ✅ Registered api-3 at {}", api3.url());
    println!();

    // 2. Discover services
    println!("2. Discovering Services:");
    let instances = discovery.discover("api").await?;
    println!("   Found {} instances of 'api' service:", instances.len());
    for instance in &instances {
        println!(
            "     - {} at {} (tags: {:?})",
            instance.id,
            instance.url(),
            instance.tags
        );
    }
    println!();

    // 3. Get specific service
    println!("3. Get Specific Service:");
    let service = discovery.get_service("api-1").await?;
    println!("   Retrieved: {}", service.id);
    println!("   Address: {}", service.url());
    println!("   Tags: {:?}", service.tags);
    println!("   Metadata: {:?}", service.metadata);
    println!();

    // 4. List all services
    println!("4. List All Services:");
    let service_names = discovery.list_services().await?;
    println!("   Registered services:");
    for name in &service_names {
        println!("     - {}", name);
    }
    println!();

    // 5. Service resolver with round-robin
    println!("5. Service Resolver (Round-Robin):");
    let resolver = ServiceResolver::new(discovery.clone(), LoadBalancingStrategy::RoundRobin);

    for i in 1..=6 {
        let instance = resolver.resolve("api").await?;
        println!("   Request {}: Routed to {}", i, instance.id);
    }
    println!();

    // 6. Service resolver with random
    println!("6. Service Resolver (Random):");
    let random_resolver = ServiceResolver::new(discovery.clone(), LoadBalancingStrategy::Random);

    for i in 1..=5 {
        let instance = random_resolver.resolve("api").await?;
        println!("   Request {}: Routed to {}", i, instance.id);
    }
    println!();

    // 7. Deregister a service
    println!("7. Deregister Service:");
    discovery.deregister("api-2").await?;
    println!("   ✅ Deregistered api-2");

    let remaining = discovery.discover("api").await?;
    println!("   Remaining instances: {}", remaining.len());
    for instance in &remaining {
        println!("     - {}", instance.id);
    }
    println!();

    // 8. Register different service type
    println!("8. Multiple Service Types:");
    let db = ServiceInstance::new("db-1", "database", "192.168.1.20", 5432)
        .with_tag("postgres")
        .with_tag("primary")
        .with_metadata("version", "14.5");

    discovery.register(&db).await?;
    println!("   ✅ Registered database service");

    let all_services = discovery.list_services().await?;
    println!("   Total service types: {}", all_services.len());
    for svc in &all_services {
        let instances = discovery.discover(svc).await?;
        println!("     - {}: {} instance(s)", svc, instances.len());
    }
    println!();

    println!("=== Service Discovery Example Complete ===\n");
    println!("💡 Key Features Demonstrated:");
    println!("   ✅ Service registration");
    println!("   ✅ Service discovery by name");
    println!("   ✅ Service metadata and tags");
    println!("   ✅ Health check URLs");
    println!("   ✅ Load balancing strategies");
    println!("   ✅ Service deregistration");
    println!("   ✅ Multiple service types");
    println!();
    println!("💡 Load Balancing Strategies:");
    println!("   - Round-Robin: Distributes evenly");
    println!("   - Random: Random selection");
    println!("   - First: Always picks first available");
    println!();
    println!("💡 Production Backends:");
    println!("   - ConsulDiscovery: Use HashiCorp Consul");
    println!("   - EtcdDiscovery: Use etcd");
    println!("   - InMemoryDiscovery: Testing/development only");
    println!();

    Ok(())
}
