//! # Implementation of the `HasValidate` trait for the `Form` extractor.
//!

use crate::HasValidate;
use axum_extra::protobuf::Protobuf;
use validator::Validate;

impl<T: Validate> HasValidate for Protobuf<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
