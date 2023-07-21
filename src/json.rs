//! # Implementation of the `HasValidate` trait for the `Json` extractor.
//!

use crate::{HasValidate, ValidRejection};
use axum::extract::rejection::JsonRejection;
use axum::Json;
use validator::Validate;

impl<T: Validate> HasValidate for Json<T> {
    type Validate = T;
    type Rejection = JsonRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl From<JsonRejection> for ValidRejection<JsonRejection> {
    fn from(value: JsonRejection) -> Self {
        Self::Inner(value)
    }
}
