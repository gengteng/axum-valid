//! # Implementation of the `HasValidate` trait for the `Yaml` extractor.
//!

use crate::HasValidate;
use axum_yaml::Yaml;
use validator::Validate;

impl<T: Validate> HasValidate for Yaml<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
