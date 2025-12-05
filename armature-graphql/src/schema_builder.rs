// Programmatic schema builder

use async_graphql::Schema;

/// Programmatic schema builder with DI integration
pub struct ProgrammaticSchemaBuilder<Q, M, S> {
    query: Option<Q>,
    mutation: Option<M>,
    subscription: Option<S>,
    services: Vec<Box<dyn std::any::Any + Send + Sync>>,
}

impl<Q, M, S> ProgrammaticSchemaBuilder<Q, M, S>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    /// Create a new schema builder
    pub fn new() -> Self {
        Self {
            query: None,
            mutation: None,
            subscription: None,
            services: Vec::new(),
        }
    }

    /// Set the query root
    pub fn query(mut self, query: Q) -> Self {
        self.query = Some(query);
        self
    }

    /// Set the mutation root
    pub fn mutation(mut self, mutation: M) -> Self {
        self.mutation = Some(mutation);
        self
    }

    /// Set the subscription root
    pub fn subscription(mut self, subscription: S) -> Self {
        self.subscription = Some(subscription);
        self
    }

    /// Add a service to the schema context
    pub fn add_service<T: Send + Sync + 'static>(mut self, service: T) -> Self {
        self.services.push(Box::new(service));
        self
    }

    /// Build the schema
    pub fn build(self) -> Schema<Q, M, S> {
        let query = self.query.expect("Query root is required");
        let mutation = self.mutation.expect("Mutation root is required");
        let subscription = self.subscription.expect("Subscription root is required");

        let mut schema_builder = Schema::build(query, mutation, subscription);

        // Add all services to the schema data
        for service in self.services {
            schema_builder = schema_builder.data(service);
        }

        schema_builder.finish()
    }
}

impl<Q, M, S> Default for ProgrammaticSchemaBuilder<Q, M, S>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create an empty query root
#[derive(Default)]
pub struct EmptyQuery;

#[async_graphql::Object]
impl EmptyQuery {
    async fn _empty(&self) -> &str {
        ""
    }
}

/// Macro to merge multiple resolver objects into a single root
#[macro_export]
macro_rules! merge_resolvers {
    ($($resolver:ty),+ $(,)?) => {
        {
            use async_graphql::MergedObject;

            #[derive(MergedObject, Default)]
            struct MergedRoot($($resolver),+);

            MergedRoot::default()
        }
    };
}

/// Helper for creating schema with merged resolvers
pub fn create_merged_schema<Q, M, S>(
    _query_resolvers: Vec<Q>,
    _mutation_resolvers: Vec<M>,
    _subscription_resolvers: Vec<S>,
) -> ProgrammaticSchemaBuilder<Q, M, S>
where
    Q: async_graphql::ObjectType + 'static,
    M: async_graphql::ObjectType + 'static,
    S: async_graphql::SubscriptionType + 'static,
{
    // Note: This is a simplified version. In practice, you'd need to merge the resolvers
    // For now, users can use the MergedObject derive macro directly
    ProgrammaticSchemaBuilder::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct TestQuery;

    #[async_graphql::Object]
    impl TestQuery {
        async fn test(&self) -> &str {
            "test"
        }
    }

    #[test]
    fn test_schema_builder() {
        let _schema = ProgrammaticSchemaBuilder::new()
            .query(TestQuery::default())
            .mutation(EmptyMutation)
            .subscription(EmptySubscription)
            .build();
    }
}
