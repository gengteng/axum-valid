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
    use crate::Valid;
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

    impl<T: ValidTest, R> ValidTest for WithRejection<Valid<T>, R> {
        // just use `418 I'm a teapot` to test
        const ERROR_STATUS_CODE: StatusCode = StatusCode::IM_A_TEAPOT;
        // If `WithRejection` is the outermost extractor,
        // the error code returned will always be the one provided by WithRejection.
        const INVALID_STATUS_CODE: StatusCode = StatusCode::IM_A_TEAPOT;
        // If `WithRejection` is the outermost extractor,
        // the returned body may not be in JSON format.
        const JSON_SERIALIZABLE: bool = false;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_valid_request(builder)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // invalid requests will cause the Valid extractor to fail.
            T::set_invalid_request(builder)
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            T::set_invalid_request(builder)
        }
    }
}
