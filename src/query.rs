//! # Implementation of the `HasValidate` trait for the `Query` extractor.
//!

use crate::{HasValidate, ValidRejection};
use axum::extract::rejection::QueryRejection;
use axum::extract::Query;
use validator::Validate;

impl<T: Validate> HasValidate for Query<T> {
    type Validate = T;
    type Rejection = QueryRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl From<QueryRejection> for ValidRejection<QueryRejection> {
    fn from(value: QueryRejection) -> Self {
        Self::Inner(value)
    }
}
