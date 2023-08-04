//! # Implementation of the `HasValidate` trait for the `Json` extractor.
//!

use crate::HasValidate;
use axum::Json;
use validator::Validate;

impl<T: Validate> HasValidate for Json<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum::Json;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for Json<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.json(T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.json(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.json(T::invalid())
        }
    }
}
