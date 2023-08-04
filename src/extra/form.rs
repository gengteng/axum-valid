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

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum_extra::extract::Form;
    use reqwest::{RequestBuilder, StatusCode};

    impl<T: ValidTestParameter> ValidTest for Form<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

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
