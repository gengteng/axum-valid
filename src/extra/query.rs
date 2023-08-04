//! # Implementation of the `HasValidate` trait for the `Query` extractor in `axum-extra`.
//!

use crate::HasValidate;
use axum_extra::extract::Query;
use validator::Validate;

impl<T: Validate> HasValidate for Query<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
