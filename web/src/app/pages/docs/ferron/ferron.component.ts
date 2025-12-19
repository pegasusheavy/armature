import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-doc-ferron',
  standalone: true,
  imports: [CommonModule, RouterLink],
  templateUrl: './ferron.component.html',
  styleUrl: './ferron.component.scss'
})
export class DocFerronComponent {
  features = [
    { icon: '⚙️', title: 'Configuration Generation', description: 'Generate Ferron KDL configs from Rust code' },
    { icon: '⚖️', title: 'Load Balancing', description: 'Round-robin, least connections, IP hash, weighted' },
    { icon: '🔍', title: 'Service Discovery', description: 'Dynamic backend registration and deregistration' },
    { icon: '💓', title: 'Health Checking', description: 'HTTP health checks with configurable thresholds' },
    { icon: '🔄', title: 'Process Management', description: 'Start, stop, restart, and reload Ferron' },
    { icon: '🔒', title: 'TLS Configuration', description: 'Automatic and manual certificate management' },
    { icon: '⏱️', title: 'Rate Limiting', description: 'Per-location rate limit configuration' },
    { icon: '🛡️', title: 'Security Headers', description: 'Automatic security header injection' }
  ];

  loadBalanceStrategies = [
    { name: 'RoundRobin', description: 'Distribute requests evenly across backends' },
    { name: 'LeastConnections', description: 'Route to backend with fewest active connections' },
    { name: 'IpHash', description: 'Consistent routing based on client IP address' },
    { name: 'Random', description: 'Random backend selection' },
    { name: 'Weighted', description: 'Route based on backend weights' }
  ];

  apiTypes = [
    { category: 'Core Types', types: [
      { name: 'FerronConfig', description: 'Main configuration builder' },
      { name: 'Backend', description: 'Backend server configuration' },
      { name: 'LoadBalancer', description: 'Load balancing configuration' },
      { name: 'Location', description: 'Path-based routing configuration' },
      { name: 'ProxyRoute', description: 'Simplified proxy route' },
      { name: 'TlsConfig', description: 'TLS/HTTPS configuration' },
      { name: 'RateLimitConfig', description: 'Rate limiting configuration' }
    ]},
    { category: 'Service Discovery', types: [
      { name: 'ServiceRegistry', description: 'Service instance registry' },
      { name: 'ServiceInstance', description: 'Registered service instance' },
      { name: 'RegistryStats', description: 'Registry statistics' }
    ]},
    { category: 'Health Checking', types: [
      { name: 'HealthState', description: 'Health state tracker' },
      { name: 'HealthCheckConfig', description: 'Health check configuration' },
      { name: 'HealthCheckResult', description: 'Health check result' },
      { name: 'HealthStatus', description: 'Health status enum' }
    ]},
    { category: 'Process Management', types: [
      { name: 'FerronProcess', description: 'Process handle' },
      { name: 'ProcessConfig', description: 'Process configuration' },
      { name: 'ProcessStatus', description: 'Process status enum' },
      { name: 'FerronManager', description: 'High-level manager' }
    ]}
  ];

  codeExamples = {
    basicProxy: `use armature_ferron::{FerronConfig, Backend};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = FerronConfig::builder()
        .domain("api.example.com")
        .backend_url("http://localhost:3000")
        .tls_auto(true)
        .gzip(true)
        .build()?;

    let kdl = config.to_kdl()?;
    println!("{}", kdl);
    Ok(())
}`,
    loadBalancing: `use armature_ferron::{FerronConfig, Backend, LoadBalancer, LoadBalanceStrategy};

let config = FerronConfig::builder()
    .domain("api.example.com")
    .load_balancer(
        LoadBalancer::new()
            .strategy(LoadBalanceStrategy::RoundRobin)
            .backend(Backend::new("http://backend1:3000"))
            .backend(Backend::new("http://backend2:3000"))
            .backend(Backend::new("http://backend3:3000"))
            .health_check_interval(30)
            .health_check_path("/health")
    )
    .build()?;`,
    serviceDiscovery: `use armature_ferron::ServiceRegistry;

let registry = ServiceRegistry::new();

// Register service instances
let id1 = registry.register("api-service", "http://localhost:3001").await?;
let id2 = registry.register("api-service", "http://localhost:3002").await?;

// Get all instances
let instances = registry.get_instances("api-service").await;
let urls = registry.get_urls("api-service").await;

// Mark unhealthy
registry.mark_unhealthy("api-service", &id1).await?;`,
    healthCheck: `use armature_ferron::{HealthCheckConfig, HealthState};
use std::time::Duration;

let config = HealthCheckConfig::new()
    .path("/health")
    .timeout(Duration::from_secs(5))
    .interval(Duration::from_secs(30))
    .unhealthy_threshold(3)
    .healthy_threshold(2);

let health_state = Arc::new(HealthState::new(config));
let result = health_state.check_backend("http://localhost:3000").await;`,
    processManagement: `use armature_ferron::{FerronProcess, ProcessConfig};

let config = ProcessConfig::new("/usr/bin/ferron", "/etc/ferron/ferron.conf")
    .auto_restart(true)
    .max_restarts(5);

let process = FerronProcess::new(config);
process.start().await?;
process.reload().await?;  // Hot reload configuration`,
    securityHeaders: `let config = FerronConfig::builder()
    .domain("api.example.com")
    .backend_url("http://localhost:3000")
    .header("X-Frame-Options", "DENY")
    .header("X-Content-Type-Options", "nosniff")
    .header("X-XSS-Protection", "1; mode=block")
    .header("Referrer-Policy", "strict-origin-when-cross-origin")
    .header("Content-Security-Policy", "default-src 'self'")
    .build()?;`
  };
}

