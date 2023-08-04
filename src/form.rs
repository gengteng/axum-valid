//! # Implementation of the `HasValidate` trait for the `Form` extractor.
//!

use crate::HasValidate;
use axum::Form;
use validator::Validate;

impl<T: Validate> HasValidate for Form<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum::Form;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for Form<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::invalid())
        }
    }
}
