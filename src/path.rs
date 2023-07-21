//! # Implementation of the `HasValidate` trait for the `Path` extractor.
//!

use crate::{HasValidate, ValidRejection};
use axum::extract::rejection::PathRejection;
use axum::extract::Path;
use validator::Validate;

impl<T: Validate> HasValidate for Path<T> {
    type Validate = T;
    type Rejection = PathRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl From<PathRejection> for ValidRejection<PathRejection> {
    fn from(value: PathRejection) -> Self {
        Self::Inner(value)
    }
}
