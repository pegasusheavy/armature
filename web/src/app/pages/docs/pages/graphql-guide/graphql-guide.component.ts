import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-graphql-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class GraphqlGuideComponent {
  page: DocPage = {
    title: 'GraphQL Integration',
    subtitle: 'Build powerful, flexible APIs with GraphQL. Armature integrates seamlessly with async-graphql for type-safe schema definition, queries, mutations, and subscriptions.',
    icon: '◈',
    badge: 'API',
    features: [
      {
        icon: '📝',
        title: 'Type-Safe Schema',
        description: 'Define schemas with Rust types, compile-time validation'
      },
      {
        icon: '🔄',
        title: 'Subscriptions',
        description: 'Real-time updates via WebSocket subscriptions'
      },
      {
        icon: '🎮',
        title: 'Playground',
        description: 'Built-in GraphQL IDE for testing and exploration'
      },
      {
        icon: '⚡',
        title: 'DataLoader',
        description: 'Automatic N+1 query prevention with batching'
      }
    ],
    sections: [
      {
        id: 'installation',
        title: 'Installation',
        content: `<p>Enable the GraphQL feature in your <code>Cargo.toml</code>:</p>`,
        codeBlocks: [
          {
            language: 'toml',
            filename: 'Cargo.toml',
            code: `[dependencies]
armature = { version = "0.1", features = ["graphql"] }
async-graphql = "7"`
          }
        ]
      },
      {
        id: 'basic-setup',
        title: 'Basic Setup',
        content: `<p>Create your GraphQL schema with Query and Mutation types:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'schema.rs',
            code: `use async_graphql::{Object, Schema, EmptySubscription};
use serde::{Deserialize, Serialize};

// Define your types
#[derive(Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub year: i32,
}

#[Object]
impl Book {
    async fn id(&self) -> &str { &self.id }
    async fn title(&self) -> &str { &self.title }
    async fn author(&self) -> &str { &self.author }
    async fn year(&self) -> i32 { self.year }
}

// Define queries
pub struct Query;

#[Object]
impl Query {
    async fn books(&self) -> Vec<Book> {
        vec![
            Book {
                id: "1".into(),
                title: "The Rust Programming Language".into(),
                author: "Steve Klabnik".into(),
                year: 2019,
            },
        ]
    }

    async fn book(&self, id: String) -> Option<Book> {
        // Fetch from database
        Some(Book {
            id,
            title: "Found Book".into(),
            author: "Author".into(),
            year: 2024,
        })
    }
}

// Define mutations
pub struct Mutation;

#[Object]
impl Mutation {
    async fn create_book(
        &self,
        title: String,
        author: String,
        year: i32,
    ) -> Book {
        Book {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            author,
            year,
        }
    }
}

// Build schema
pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn create_schema() -> AppSchema {
    Schema::build(Query, Mutation, EmptySubscription).finish()
}`
          }
        ]
      },
      {
        id: 'controller-setup',
        title: 'GraphQL Controller',
        content: `<p>Create a controller to handle GraphQL requests and serve the playground:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            filename: 'graphql_controller.rs',
            code: `use armature::prelude::*;
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};

#[controller("/graphql")]
#[derive(Default, Clone)]
struct GraphQLController {
    schema: AppSchema,
}

impl GraphQLController {
    #[post("")]
    async fn query(&self, req: HttpRequest) -> Result<HttpResponse, Error> {
        let body: async_graphql::Request = req.json().await?;
        let response = self.schema.execute(body).await;

        HttpResponse::ok().with_json(&response)
    }

    #[get("/playground")]
    async fn playground(&self, _req: HttpRequest) -> Result<HttpResponse, Error> {
        let html = playground_source(
            GraphQLPlaygroundConfig::new("/graphql")
        );

        Ok(HttpResponse::ok()
            .with_header("Content-Type", "text/html")
            .with_body(html))
    }
}`
          }
        ]
      },
      {
        id: 'subscriptions',
        title: 'Real-time Subscriptions',
        content: `<p>Add WebSocket subscriptions for real-time updates:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use async_graphql::{Subscription, Object};
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;

pub struct Subscription;

#[Subscription]
impl Subscription {
    async fn book_added(&self) -> impl Stream<Item = Book> {
        // Return a stream that yields new books
        BroadcastStream::new(book_channel.subscribe())
            .filter_map(|result| result.ok())
    }

    async fn counter(&self, start: i32) -> impl Stream<Item = i32> {
        let mut value = start;
        async_stream::stream! {
            loop {
                yield value;
                value += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}`
          }
        ]
      },
      {
        id: 'dataloader',
        title: 'DataLoader for N+1 Prevention',
        content: `<p>Use DataLoader to batch database queries and prevent N+1 problems:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use async_graphql::dataloader::{DataLoader, Loader};
use std::collections::HashMap;

struct AuthorLoader {
    db: DatabaseService,
}

impl Loader<String> for AuthorLoader {
    type Value = Author;
    type Error = Error;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Author>, Error> {
        // Batch load all authors in a single query
        let authors = self.db.find_authors_by_ids(keys).await?;
        Ok(authors.into_iter().map(|a| (a.id.clone(), a)).collect())
    }
}

// Use in your schema
#[Object]
impl Book {
    async fn author(&self, ctx: &Context<'_>) -> Result<Author, Error> {
        let loader = ctx.data::<DataLoader<AuthorLoader>>()?;
        loader.load_one(self.author_id.clone()).await?.ok_or(Error::NotFound)
    }
}`
          }
        ]
      },
      {
        id: 'authentication',
        title: 'Authentication & Guards',
        content: `<p>Protect queries and mutations with guards:</p>`,
        codeBlocks: [
          {
            language: 'rust',
            code: `use async_graphql::{guard, Context};

struct RoleGuard {
    role: String,
}

#[async_trait::async_trait]
impl Guard for RoleGuard {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        let user = ctx.data::<CurrentUser>()?;
        if user.has_role(&self.role) {
            Ok(())
        } else {
            Err("Forbidden".into())
        }
    }
}

#[Object]
impl Mutation {
    #[graphql(guard = "RoleGuard { role: \"admin\".into() }")]
    async fn delete_book(&self, id: String) -> Result<bool, Error> {
        // Only admins can delete books
        Ok(true)
    }
}`
          }
        ]
      },
      {
        id: 'example-queries',
        title: 'Example Queries',
        content: `<p>Try these queries in the GraphQL Playground:</p>`,
        subsections: [
          {
            id: 'query-examples',
            title: 'Query Examples',
            codeBlocks: [
              {
                language: 'graphql',
                filename: 'Get all books',
                code: `query {
  books {
    id
    title
    author
    year
  }
}`
              },
              {
                language: 'graphql',
                filename: 'Get book by ID',
                code: `query {
  book(id: "1") {
    id
    title
    author
    year
  }
}`
              }
            ]
          },
          {
            id: 'mutation-examples',
            title: 'Mutation Examples',
            codeBlocks: [
              {
                language: 'graphql',
                filename: 'Create a new book',
                code: `mutation {
  createBook(
    title: "Zero to Production in Rust"
    author: "Luca Palmieri"
    year: 2022
  ) {
    id
    title
  }
}`
              }
            ]
          }
        ]
      }
    ],
    relatedDocs: [
      {
        id: 'graphql-config',
        title: 'Advanced GraphQL Configuration',
        description: 'Caching, complexity limits, and extensions'
      },
      {
        id: 'websocket-sse',
        title: 'WebSockets & SSE',
        description: 'Real-time communication patterns'
      },
      {
        id: 'auth-guide',
        title: 'Authentication Guide',
        description: 'Secure your GraphQL API'
      }
    ],
    seeAlso: [
      { title: 'OpenAPI Documentation', id: 'openapi-guide' },
      { title: 'API Versioning', id: 'api-versioning' },
      { title: 'Content Negotiation', id: 'content-negotiation' }
    ]
  };
}

