//! # Implementation of the `HasValidate` trait for the `Form` extractor in `axum-extra`.
//!

use crate::HasValidate;
use axum_extra::extract::Form;
use validator::Validate;

impl<T: Validate> HasValidate for Form<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
