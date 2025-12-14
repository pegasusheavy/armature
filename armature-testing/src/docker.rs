//! Docker Test Containers
//!
//! Provides Docker-based testing with automatic container lifecycle management.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use thiserror::Error;

/// Docker test container errors
#[derive(Debug, Error)]
pub enum DockerError {
    #[error("Docker not available: {0}")]
    NotAvailable(String),

    #[error("Container start failed: {0}")]
    StartFailed(String),

    #[error("Container stop failed: {0}")]
    StopFailed(String),

    #[error("Image pull failed: {0}")]
    PullFailed(String),

    #[error("Container not found: {0}")]
    NotFound(String),
}

/// Docker container configuration
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker image
    pub image: String,

    /// Image tag
    pub tag: String,

    /// Container name
    pub name: Option<String>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Port mappings (host_port -> container_port)
    pub ports: HashMap<u16, u16>,

    /// Wait for container to be ready
    pub wait_timeout_secs: u64,
}

impl ContainerConfig {
    /// Create new container config
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_testing::docker::ContainerConfig;
    ///
    /// let config = ContainerConfig::new("postgres", "15");
    /// ```
    pub fn new(image: impl Into<String>, tag: impl Into<String>) -> Self {
        Self {
            image: image.into(),
            tag: tag.into(),
            name: None,
            env: HashMap::new(),
            ports: HashMap::new(),
            wait_timeout_secs: 30,
        }
    }

    /// Set container name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Add port mapping
    pub fn with_port(mut self, host_port: u16, container_port: u16) -> Self {
        self.ports.insert(host_port, container_port);
        self
    }

    /// Set wait timeout
    pub fn with_wait_timeout(mut self, seconds: u64) -> Self {
        self.wait_timeout_secs = seconds;
        self
    }

    /// Get full image name
    pub fn image_name(&self) -> String {
        format!("{}:{}", self.image, self.tag)
    }
}

/// Docker test container
pub struct DockerContainer {
    config: ContainerConfig,
    container_id: Option<String>,
}

impl DockerContainer {
    /// Create new Docker container
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_testing::docker::*;
    ///
    /// let config = ContainerConfig::new("postgres", "15")
    ///     .with_env("POSTGRES_PASSWORD", "test")
    ///     .with_port(5432, 5432);
    ///
    /// let container = DockerContainer::new(config);
    /// ```
    pub fn new(config: ContainerConfig) -> Self {
        Self {
            config,
            container_id: None,
        }
    }

    /// Check if Docker is available
    pub fn is_docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Start container
    pub async fn start(&mut self) -> Result<(), DockerError> {
        if !Self::is_docker_available() {
            return Err(DockerError::NotAvailable(
                "Docker not found. Please install Docker.".to_string(),
            ));
        }

        // Pull image if needed
        self.pull_image()?;

        // Build docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("-d") // Detached
            .arg("--rm"); // Auto-remove on stop

        // Container name
        if let Some(ref name) = self.config.name {
            cmd.arg("--name").arg(name);
        }

        // Environment variables
        for (key, value) in &self.config.env {
            cmd.arg("-e").arg(format!("{}={}", key, value));
        }

        // Port mappings
        for (host_port, container_port) in &self.config.ports {
            cmd.arg("-p").arg(format!("{}:{}", host_port, container_port));
        }

        // Image
        cmd.arg(self.config.image_name());

        // Execute
        let output = cmd
            .output()
            .map_err(|e| DockerError::StartFailed(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::StartFailed(error.to_string()));
        }

        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        self.container_id = Some(container_id);

        // Wait for container to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        Ok(())
    }

    /// Stop container
    pub async fn stop(&mut self) -> Result<(), DockerError> {
        if let Some(ref container_id) = self.container_id {
            let output = Command::new("docker")
                .arg("stop")
                .arg(container_id)
                .output()
                .map_err(|e| DockerError::StopFailed(e.to_string()))?;

            if !output.status.success() {
                let error = String::from_utf8_lossy(&output.stderr);
                return Err(DockerError::StopFailed(error.to_string()));
            }

            self.container_id = None;
        }

        Ok(())
    }

    /// Pull Docker image
    fn pull_image(&self) -> Result<(), DockerError> {
        let output = Command::new("docker")
            .arg("pull")
            .arg(self.config.image_name())
            .output()
            .map_err(|e| DockerError::PullFailed(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(DockerError::PullFailed(error.to_string()));
        }

        Ok(())
    }

    /// Get container ID
    pub fn container_id(&self) -> Option<&str> {
        self.container_id.as_deref()
    }

    /// Check if container is running
    pub fn is_running(&self) -> bool {
        if let Some(ref container_id) = self.container_id {
            Command::new("docker")
                .arg("inspect")
                .arg("-f")
                .arg("{{.State.Running}}")
                .arg(container_id)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim() == "true")
                .unwrap_or(false)
        } else {
            false
        }
    }
}

impl Drop for DockerContainer {
    fn drop(&mut self) {
        // Stop container on drop
        if self.container_id.is_some() {
            let _ = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(self.stop());
        }
    }
}

/// Postgres test container helper
pub struct PostgresContainer;

impl PostgresContainer {
    /// Create Postgres test container
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_testing::docker::PostgresContainer;
    ///
    /// let config = PostgresContainer::config("testdb", "testuser", "testpass");
    /// ```
    pub fn config(database: &str, username: &str, password: &str) -> ContainerConfig {
        ContainerConfig::new("postgres", "15")
            .with_name(format!("armature-test-postgres-{}", uuid::Uuid::new_v4()))
            .with_env("POSTGRES_DB", database)
            .with_env("POSTGRES_USER", username)
            .with_env("POSTGRES_PASSWORD", password)
            .with_port(5432, 5432)
            .with_wait_timeout(10)
    }
}

/// Redis test container helper
pub struct RedisContainer;

impl RedisContainer {
    /// Create Redis test container
    ///
    /// # Examples
    ///
    /// ```
    /// use armature_testing::docker::RedisContainer;
    ///
    /// let config = RedisContainer::config();
    /// ```
    pub fn config() -> ContainerConfig {
        ContainerConfig::new("redis", "7")
            .with_name(format!("armature-test-redis-{}", uuid::Uuid::new_v4()))
            .with_port(6379, 6379)
            .with_wait_timeout(5)
    }
}

/// MongoDB test container helper
pub struct MongoContainer;

impl MongoContainer {
    /// Create MongoDB test container
    pub fn config(database: &str) -> ContainerConfig {
        ContainerConfig::new("mongo", "7")
            .with_name(format!("armature-test-mongo-{}", uuid::Uuid::new_v4()))
            .with_env("MONGO_INITDB_DATABASE", database)
            .with_port(27017, 27017)
            .with_wait_timeout(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_config() {
        let config = ContainerConfig::new("postgres", "15")
            .with_name("test-db")
            .with_env("POSTGRES_PASSWORD", "test")
            .with_port(5432, 5432);

        assert_eq!(config.image, "postgres");
        assert_eq!(config.tag, "15");
        assert_eq!(config.name, Some("test-db".to_string()));
        assert_eq!(config.env.get("POSTGRES_PASSWORD"), Some(&"test".to_string()));
        assert_eq!(config.ports.get(&5432), Some(&5432));
    }

    #[test]
    fn test_postgres_container_config() {
        let config = PostgresContainer::config("testdb", "user", "pass");
        assert_eq!(config.image, "postgres");
        assert_eq!(config.env.get("POSTGRES_DB"), Some(&"testdb".to_string()));
    }

    #[test]
    fn test_redis_container_config() {
        let config = RedisContainer::config();
        assert_eq!(config.image, "redis");
        assert_eq!(config.tag, "7");
    }

    #[test]
    fn test_docker_available() {
        // This will vary by system
        let _ = DockerContainer::is_docker_available();
    }
}

