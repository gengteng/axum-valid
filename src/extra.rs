//! # Implementation of the `HasValidate` trait for the extractor in `axum-extra`.
//!

#[cfg(feature = "extra_form")]
pub mod form;
#[cfg(feature = "extra_protobuf")]
pub mod protobuf;
#[cfg(feature = "extra_query")]
pub mod query;

use crate::HasValidate;
use axum_extra::extract::{Cached, WithRejection};
use validator::Validate;

impl<T: Validate> HasValidate for Cached<T> {
    type Validate = T;

    fn get_validate(&self) -> &Self::Validate {
        &self.0
    }
}

impl<T: Validate, R> HasValidate for WithRejection<T, R> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{Rejection, ValidTest};
    use axum::http::StatusCode;
    use axum_extra::extract::{Cached, WithRejection};
    use reqwest::RequestBuilder;

    impl<T: ValidTest> ValidTest for Cached<T> {
        const ERROR_STATUS_CODE: StatusCode = T::ERROR_STATUS_CODE;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // cached never fails
            T::set_error_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }

    impl<T: ValidTest, R: Rejection> ValidTest for WithRejection<T, R> {
        // just use conflict to test
        const ERROR_STATUS_CODE: StatusCode = R::STATUS_CODE;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // cached never fails
            T::set_error_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }
}
