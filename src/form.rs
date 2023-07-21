//! # Implementation of the `HasValidate` trait for the `Form` extractor.
//!

use crate::{HasValidate, ValidRejection};
use axum::extract::rejection::FormRejection;
use axum::Form;
use validator::Validate;

impl<T: Validate> HasValidate for Form<T> {
    type Validate = T;
    type Rejection = FormRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl From<FormRejection> for ValidRejection<FormRejection> {
    fn from(value: FormRejection) -> Self {
        Self::Inner(value)
    }
}
