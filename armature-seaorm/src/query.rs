//! Query builder utilities for SeaORM.

use sea_orm::{ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, QuerySelect, Select};
use serde::Deserialize;

/// Query builder for common query patterns.
pub struct QueryBuilder<E: EntityTrait> {
    select: Option<Select<E>>,
    conditions: Vec<Box<dyn Fn(Condition) -> Condition + Send + Sync>>,
    #[allow(dead_code)] // Reserved for future use
    orders: Vec<(String, SortOrder)>,
    limit: Option<u64>,
    offset: Option<u64>,
}

impl<E: EntityTrait> Default for QueryBuilder<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending order.
    #[default]
    Asc,
    /// Descending order.
    Desc,
}

impl<E: EntityTrait> QueryBuilder<E> {
    /// Create a new query builder.
    pub fn new() -> Self {
        Self {
            select: Some(E::find()),
            conditions: Vec::new(),
            orders: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    /// Add a limit.
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add an offset.
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build the final Select query.
    pub fn build(mut self) -> Select<E> {
        let mut select = self.select.take().unwrap_or_else(|| E::find());

        // Apply conditions
        if !self.conditions.is_empty() {
            let mut condition = Condition::all();
            for f in self.conditions {
                condition = f(condition);
            }
            select = select.filter(condition);
        }

        // Apply limit and offset
        if let Some(limit) = self.limit {
            select = select.limit(limit);
        }
        if let Some(offset) = self.offset {
            select = select.offset(offset);
        }

        select
    }
}

/// Extension trait for query helpers.
pub trait QueryExt<E: EntityTrait>: Sized {
    /// Add a where clause for equality.
    fn where_eq<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for inequality.
    fn where_ne<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for greater than.
    fn where_gt<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for greater than or equal.
    fn where_gte<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for less than.
    fn where_lt<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for less than or equal.
    fn where_lte<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self;

    /// Add a where clause for LIKE.
    fn where_like<C: ColumnTrait>(self, column: C, pattern: &str) -> Self;

    /// Add a where clause for IS NULL.
    fn where_null<C: ColumnTrait>(self, column: C) -> Self;

    /// Add a where clause for IS NOT NULL.
    fn where_not_null<C: ColumnTrait>(self, column: C) -> Self;

    /// Add a where clause for IN.
    fn where_in<C: ColumnTrait, I: IntoIterator<Item = V>, V: Into<sea_orm::Value>>(
        self,
        column: C,
        values: I,
    ) -> Self;

    /// Add a where clause for BETWEEN.
    fn where_between<C: ColumnTrait, V: Into<sea_orm::Value>>(
        self,
        column: C,
        low: V,
        high: V,
    ) -> Self;

    /// Order by ascending.
    fn order_asc<C: ColumnTrait>(self, column: C) -> Self;

    /// Order by descending.
    fn order_desc<C: ColumnTrait>(self, column: C) -> Self;
}

impl<E: EntityTrait> QueryExt<E> for Select<E> {
    fn where_eq<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.eq(value))
    }

    fn where_ne<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.ne(value))
    }

    fn where_gt<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.gt(value))
    }

    fn where_gte<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.gte(value))
    }

    fn where_lt<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.lt(value))
    }

    fn where_lte<C: ColumnTrait>(self, column: C, value: impl Into<sea_orm::Value>) -> Self {
        self.filter(column.lte(value))
    }

    fn where_like<C: ColumnTrait>(self, column: C, pattern: &str) -> Self {
        self.filter(column.like(pattern))
    }

    fn where_null<C: ColumnTrait>(self, column: C) -> Self {
        self.filter(column.is_null())
    }

    fn where_not_null<C: ColumnTrait>(self, column: C) -> Self {
        self.filter(column.is_not_null())
    }

    fn where_in<C: ColumnTrait, I: IntoIterator<Item = V>, V: Into<sea_orm::Value>>(
        self,
        column: C,
        values: I,
    ) -> Self {
        self.filter(column.is_in(values))
    }

    fn where_between<C: ColumnTrait, V: Into<sea_orm::Value>>(
        self,
        column: C,
        low: V,
        high: V,
    ) -> Self {
        self.filter(column.between(low, high))
    }

    fn order_asc<C: ColumnTrait>(self, column: C) -> Self {
        self.order_by_asc(column)
    }

    fn order_desc<C: ColumnTrait>(self, column: C) -> Self {
        self.order_by_desc(column)
    }
}

/// Search filters parsed from query parameters.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct SearchFilters {
    /// Text search query.
    #[serde(default)]
    pub q: Option<String>,

    /// Sort field.
    #[serde(default)]
    pub sort: Option<String>,

    /// Sort order.
    #[serde(default)]
    pub order: SortOrder,

    /// Page number.
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

impl SearchFilters {
    /// Get pagination options.
    pub fn pagination(&self) -> crate::PaginationOptions {
        crate::PaginationOptions::new(self.page, self.per_page)
    }
}

