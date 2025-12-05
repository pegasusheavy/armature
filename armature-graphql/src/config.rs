/// GraphQL server configuration
#[derive(Debug, Clone)]
pub struct GraphQLConfig {
    /// GraphQL endpoint path
    pub endpoint: String,
    
    /// Enable GraphQL Playground (interactive GraphQL IDE)
    pub enable_playground: bool,
    
    /// Playground endpoint path (if enabled)
    pub playground_endpoint: String,
    
    /// Enable GraphiQL (lighter alternative to Playground)
    pub enable_graphiql: bool,
    
    /// GraphiQL endpoint path (if enabled)
    pub graphiql_endpoint: String,
    
    /// Enable schema documentation endpoint
    pub enable_schema_docs: bool,
    
    /// Schema documentation endpoint path
    pub schema_docs_endpoint: String,
    
    /// Enable introspection queries (required for playgrounds and docs)
    pub enable_introspection: bool,
    
    /// Maximum query depth (0 = unlimited)
    pub max_depth: usize,
    
    /// Maximum query complexity (0 = unlimited)
    pub max_complexity: usize,
    
    /// Enable query validation
    pub enable_validation: bool,
    
    /// Enable Apollo Tracing
    pub enable_tracing: bool,
}

impl GraphQLConfig {
    /// Create a new GraphQL configuration with defaults
    ///
    /// # Example
    ///
    /// ```
    /// use armature_graphql::GraphQLConfig;
    ///
    /// let config = GraphQLConfig::new("/graphql");
    /// assert_eq!(config.endpoint, "/graphql");
    /// assert!(config.enable_playground); // Enabled by default in development
    /// ```
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            playground_endpoint: format!("{}/playground", endpoint),
            graphiql_endpoint: format!("{}/graphiql", endpoint),
            schema_docs_endpoint: format!("{}/schema", endpoint),
            endpoint,
            enable_playground: true,
            enable_graphiql: false,
            enable_schema_docs: true,
            enable_introspection: true,
            max_depth: 0,
            max_complexity: 0,
            enable_validation: true,
            enable_tracing: false,
        }
    }

    /// Create a production configuration (playgrounds disabled)
    ///
    /// # Example
    ///
    /// ```
    /// use armature_graphql::GraphQLConfig;
    ///
    /// let config = GraphQLConfig::production("/graphql");
    /// assert!(!config.enable_playground);
    /// assert!(!config.enable_graphiql);
    /// assert!(!config.enable_introspection); // Disabled for security
    /// ```
    pub fn production(endpoint: impl Into<String>) -> Self {
        let mut config = Self::new(endpoint);
        config.enable_playground = false;
        config.enable_graphiql = false;
        config.enable_introspection = false;
        config.enable_schema_docs = false; // Can be enabled separately if needed
        config
    }

    /// Create a development configuration (all features enabled)
    ///
    /// # Example
    ///
    /// ```
    /// use armature_graphql::GraphQLConfig;
    ///
    /// let config = GraphQLConfig::development("/graphql");
    /// assert!(config.enable_playground);
    /// assert!(config.enable_graphiql);
    /// assert!(config.enable_schema_docs);
    /// assert!(config.enable_introspection);
    /// ```
    pub fn development(endpoint: impl Into<String>) -> Self {
        let mut config = Self::new(endpoint);
        config.enable_playground = true;
        config.enable_graphiql = true;
        config.enable_schema_docs = true;
        config.enable_introspection = true;
        config.enable_tracing = true;
        config
    }

    /// Enable or disable GraphQL Playground
    pub fn with_playground(mut self, enable: bool) -> Self {
        self.enable_playground = enable;
        self
    }

    /// Set custom playground endpoint
    pub fn with_playground_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.playground_endpoint = endpoint.into();
        self
    }

    /// Enable or disable GraphiQL
    pub fn with_graphiql(mut self, enable: bool) -> Self {
        self.enable_graphiql = enable;
        self
    }

    /// Set custom GraphiQL endpoint
    pub fn with_graphiql_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.graphiql_endpoint = endpoint.into();
        self
    }

    /// Enable or disable schema documentation endpoint
    pub fn with_schema_docs(mut self, enable: bool) -> Self {
        self.enable_schema_docs = enable;
        self
    }

    /// Set custom schema documentation endpoint
    pub fn with_schema_docs_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.schema_docs_endpoint = endpoint.into();
        self
    }

    /// Enable or disable introspection queries
    pub fn with_introspection(mut self, enable: bool) -> Self {
        self.enable_introspection = enable;
        self
    }

    /// Set maximum query depth
    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set maximum query complexity
    pub fn with_max_complexity(mut self, complexity: usize) -> Self {
        self.max_complexity = complexity;
        self
    }

    /// Enable or disable query validation
    pub fn with_validation(mut self, enable: bool) -> Self {
        self.enable_validation = enable;
        self
    }

    /// Enable or disable Apollo Tracing
    pub fn with_tracing(mut self, enable: bool) -> Self {
        self.enable_tracing = enable;
        self
    }
}

impl Default for GraphQLConfig {
    fn default() -> Self {
        Self::new("/graphql")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GraphQLConfig::new("/graphql");
        assert_eq!(config.endpoint, "/graphql");
        assert!(config.enable_playground);
        assert!(config.enable_introspection);
        assert!(config.enable_schema_docs);
    }

    #[test]
    fn test_production_config() {
        let config = GraphQLConfig::production("/api/graphql");
        assert_eq!(config.endpoint, "/api/graphql");
        assert!(!config.enable_playground);
        assert!(!config.enable_graphiql);
        assert!(!config.enable_introspection);
        assert!(!config.enable_schema_docs);
    }

    #[test]
    fn test_development_config() {
        let config = GraphQLConfig::development("/dev/graphql");
        assert_eq!(config.endpoint, "/dev/graphql");
        assert!(config.enable_playground);
        assert!(config.enable_graphiql);
        assert!(config.enable_schema_docs);
        assert!(config.enable_introspection);
        assert!(config.enable_tracing);
    }

    #[test]
    fn test_builder_pattern() {
        let config = GraphQLConfig::new("/graphql")
            .with_playground(false)
            .with_graphiql(true)
            .with_schema_docs(true)
            .with_max_depth(10)
            .with_max_complexity(100);

        assert!(!config.enable_playground);
        assert!(config.enable_graphiql);
        assert!(config.enable_schema_docs);
        assert_eq!(config.max_depth, 10);
        assert_eq!(config.max_complexity, 100);
    }

    #[test]
    fn test_custom_endpoints() {
        let config = GraphQLConfig::new("/api")
            .with_playground_endpoint("/api/play")
            .with_graphiql_endpoint("/api/iql")
            .with_schema_docs_endpoint("/api/docs");

        assert_eq!(config.playground_endpoint, "/api/play");
        assert_eq!(config.graphiql_endpoint, "/api/iql");
        assert_eq!(config.schema_docs_endpoint, "/api/docs");
    }
}

