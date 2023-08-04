//! # Implementation of the `HasValidate` trait for the `Query` extractor.
//!

use crate::HasValidate;
use axum::extract::Query;
use validator::Validate;

impl<T: Validate> HasValidate for Query<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::extract::Query;
    use axum::http::StatusCode;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for Query<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(&T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.query(&T::invalid())
        }
    }
}
