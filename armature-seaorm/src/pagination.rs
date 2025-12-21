//! Pagination utilities for SeaORM queries.

use sea_orm::{entity::prelude::*, EntityTrait, PaginatorTrait, Select};
use serde::{Deserialize, Serialize};

/// Pagination options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationOptions {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    pub page: u64,

    /// Items per page.
    #[serde(default = "default_per_page")]
    pub per_page: u64,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    20
}

impl Default for PaginationOptions {
    fn default() -> Self {
        Self {
            page: default_page(),
            per_page: default_per_page(),
        }
    }
}

impl PaginationOptions {
    /// Create new pagination options.
    pub fn new(page: u64, per_page: u64) -> Self {
        Self { page, per_page }
    }

    /// Get the offset for SQL queries.
    pub fn offset(&self) -> u64 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Get the limit for SQL queries.
    pub fn limit(&self) -> u64 {
        self.per_page
    }
}

/// A paginated result set.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    /// The items in this page.
    pub items: Vec<T>,

    /// Pagination metadata.
    pub meta: PaginationMeta,
}

/// Pagination metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page number.
    pub page: u64,

    /// Items per page.
    pub per_page: u64,

    /// Total number of items.
    pub total_items: u64,

    /// Total number of pages.
    pub total_pages: u64,

    /// Whether there is a next page.
    pub has_next: bool,

    /// Whether there is a previous page.
    pub has_prev: bool,
}

impl<T> Paginated<T> {
    /// Create a new paginated result.
    pub fn new(items: Vec<T>, page: u64, per_page: u64, total_items: u64) -> Self {
        let total_pages = total_items.div_ceil(per_page);

        Self {
            items,
            meta: PaginationMeta {
                page,
                per_page,
                total_items,
                total_pages,
                has_next: page < total_pages,
                has_prev: page > 1,
            },
        }
    }

    /// Map items to a different type.
    pub fn map<U, F>(self, f: F) -> Paginated<U>
    where
        F: FnMut(T) -> U,
    {
        Paginated {
            items: self.items.into_iter().map(f).collect(),
            meta: self.meta,
        }
    }
}

/// Extension trait for paginated queries.
#[async_trait::async_trait]
pub trait Paginate<E>
where
    E: EntityTrait,
{
    /// Paginate the query.
    async fn paginate(
        self,
        db: &impl sea_orm::ConnectionTrait,
        options: &PaginationOptions,
    ) -> Result<Paginated<E::Model>, sea_orm::DbErr>
    where
        E::Model: Sync;
}

#[async_trait::async_trait]
impl<E> Paginate<E> for Select<E>
where
    E: EntityTrait,
    E::Model: Send,
{
    async fn paginate(
        self,
        db: &impl sea_orm::ConnectionTrait,
        options: &PaginationOptions,
    ) -> Result<Paginated<E::Model>, sea_orm::DbErr>
    where
        E::Model: Sync,
    {
        let paginator = PaginatorTrait::paginate(self, db, options.per_page);
        let total_items = paginator.num_items().await?;
        let items = paginator.fetch_page(options.page.saturating_sub(1)).await?;

        Ok(Paginated::new(items, options.page, options.per_page, total_items))
    }
}

/// Cursor-based pagination options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorOptions {
    /// Cursor value (typically an ID or timestamp).
    pub cursor: Option<String>,

    /// Number of items to fetch.
    #[serde(default = "default_per_page")]
    pub limit: u64,

    /// Direction (forward or backward).
    #[serde(default)]
    pub direction: CursorDirection,
}

/// Cursor direction.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorDirection {
    /// Forward pagination.
    #[default]
    Forward,
    /// Backward pagination.
    Backward,
}

impl Default for CursorOptions {
    fn default() -> Self {
        Self {
            cursor: None,
            limit: default_per_page(),
            direction: CursorDirection::Forward,
        }
    }
}

/// Cursor-paginated result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorPaginated<T> {
    /// The items.
    pub items: Vec<T>,

    /// Next cursor for forward pagination.
    pub next_cursor: Option<String>,

    /// Previous cursor for backward pagination.
    pub prev_cursor: Option<String>,

    /// Whether there are more items.
    pub has_more: bool,
}

impl<T> CursorPaginated<T> {
    /// Create a new cursor-paginated result.
    pub fn new(
        items: Vec<T>,
        next_cursor: Option<String>,
        prev_cursor: Option<String>,
        has_more: bool,
    ) -> Self {
        Self {
            items,
            next_cursor,
            prev_cursor,
            has_more,
        }
    }
}

