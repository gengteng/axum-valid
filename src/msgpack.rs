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

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_msgpack::{MsgPack, MsgPackRaw};
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for MsgPack<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // `Content-Type` not set, `MsgPack` should return `415 Unsupported Media Type`
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }

    impl<T: ValidTestParameter> ValidTest for MsgPackRaw<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // `Content-Type` not set, `MsgPack` should return `415 Unsupported Media Type`
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }
}
