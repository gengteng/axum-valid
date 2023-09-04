//! # Implementation of the `HasValidate` trait for the `TypedMultipart` extractor.
//!

use crate::HasValidate;
use axum_typed_multipart::TypedMultipart;
use validator::Validate;

impl<T: Validate> HasValidate for TypedMultipart<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_typed_multipart::TypedMultipart;
    use reqwest::multipart::Form;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for TypedMultipart<T>
    where
        Form: From<&'static T>,
    {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::valid()))
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::new())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::invalid()))
        }
    }
}
