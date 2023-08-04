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

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_extra::protobuf::Protobuf;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter + prost::Message> ValidTest for Protobuf<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body(T::valid().encode_to_vec())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body("invalid protobuf")
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.body(T::invalid().encode_to_vec())
        }
    }
}
