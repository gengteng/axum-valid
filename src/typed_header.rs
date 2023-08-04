//! # Implementation of the `HasValidate` trait for the `TypedHeader` extractor.
//!

use crate::HasValidate;
use axum::TypedHeader;
use validator::Validate;

impl<T: Validate> HasValidate for TypedHeader<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::headers::{Header, HeaderMapExt};
    use axum::http::StatusCode;
    use axum::TypedHeader;
    use reqwest::header::HeaderMap;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter + Header + Clone> ValidTest for TypedHeader<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            let mut headers = HeaderMap::default();
            headers.typed_insert(T::valid().clone());
            builder.headers(headers)
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            let mut headers = HeaderMap::default();
            headers.typed_insert(T::invalid().clone());
            builder.headers(headers)
        }
    }
}
