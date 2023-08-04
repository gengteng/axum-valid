//! # Implementation of the `HasValidate` trait for the `MsgPack` extractor.
//!

use crate::HasValidate;
use axum_msgpack::{MsgPack, MsgPackRaw};
use validator::Validate;

impl<T: Validate> HasValidate for MsgPack<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> HasValidate for MsgPackRaw<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}
