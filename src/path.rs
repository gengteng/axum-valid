//! # Implementation of the `HasValidate` trait for the `Path` extractor.
//!

use crate::HasValidate;
use axum::extract::Path;
use validator::Validate;

impl<T: Validate> HasValidate for Path<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
