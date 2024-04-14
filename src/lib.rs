#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs, clippy::unwrap_used)]

#[cfg(feature = "extra")]
pub mod extra;
#[cfg(feature = "form")]
pub mod form;
#[cfg(feature = "garde")]
pub mod garde;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "msgpack")]
pub mod msgpack;
pub mod path;
#[cfg(feature = "query")]
pub mod query;
#[cfg(feature = "typed_header")]
pub mod typed_header;
#[cfg(feature = "validator")]
pub mod validator;
#[cfg(feature = "validify")]
pub mod validify;
#[cfg(feature = "yaml")]
pub mod yaml;

#[cfg(feature = "cbor")]
pub mod cbor;
#[cfg(feature = "sonic")]
pub mod sonic;
#[cfg(feature = "toml")]
pub mod toml;
#[cfg(feature = "typed_multipart")]
pub mod typed_multipart;
#[cfg(feature = "xml")]
pub mod xml;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error;
use std::fmt::Display;

/// Http status code returned when there are validation errors.
#[cfg(feature = "422")]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;
/// Http status code returned when there are validation errors.
#[cfg(not(feature = "422"))]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::BAD_REQUEST;

/// Trait for types that can supply a reference that can be validated.
///
/// Extractor types `T` that implement this trait can be used with `Valid`, `Garde` or `Validated`.
///
pub trait HasValidate {
    /// Inner type that can be validated for correctness
    type Validate;
    /// Get the inner value
    fn get_validate(&self) -> &Self::Validate;
}

#[cfg(feature = "validator")]
pub use crate::validator::{HasValidateArgs, Valid, ValidEx, ValidRejection};

#[cfg(feature = "garde")]
pub use crate::garde::{Garde, GardeRejection};

#[cfg(feature = "validify")]
pub use crate::validify::{
    HasModify, HasValidify, Modified, PayloadExtractor, Validated, Validified, ValidifiedByRef,
    ValidifyRejection,
};

/// `ValidationRejection` is returned when the validation extractor fails.
///
/// This enumeration captures two types of errors that can occur when using `Valid`: errors related to the validation
/// extractor itself , and errors that may arise within the inner extractor (represented by `Inner`).
///
#[derive(Debug)]
pub enum ValidationRejection<V, E> {
    /// `Valid` variant captures errors related to the validation logic.
    Valid(V),
    /// `Inner` variant represents potential errors that might occur within the inner extractor.
    Inner(E),
}

impl<V: Display, E: Display> Display for ValidationRejection<V, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationRejection::Valid(errors) => write!(f, "{errors}"),
            ValidationRejection::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<V: Error + 'static, E: Error + 'static> Error for ValidationRejection<V, E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidationRejection::Valid(ve) => Some(ve),
            ValidationRejection::Inner(e) => Some(e),
        }
    }
}

#[cfg(feature = "into_json")]
impl<V: serde::Serialize, E: IntoResponse> IntoResponse for ValidationRejection<V, E> {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::Valid(v) => {
                (VALIDATION_ERROR_STATUS, axum::Json(v)).into_response()
            }
            ValidationRejection::Inner(e) => e.into_response(),
        }
    }
}

#[cfg(not(feature = "into_json"))]
impl<V: Display, E: IntoResponse> IntoResponse for ValidationRejection<V, E> {
    fn into_response(self) -> Response {
        match self {
            ValidationRejection::Valid(v) => {
                (VALIDATION_ERROR_STATUS, v.to_string()).into_response()
            }
            ValidationRejection::Inner(e) => e.into_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use reqwest::RequestBuilder;

    /// # Valid test parameter
    pub trait ValidTestParameter: 'static {
        /// Create a valid parameter
        fn valid() -> &'static Self;
        /// Create an error serializable array
        fn error() -> &'static [(&'static str, &'static str)];
        /// Create a invalid parameter
        fn invalid() -> &'static Self;
    }

    /// # Valid Tests
    ///
    /// This trait defines three test cases to check
    /// if an extractor combined with the Valid type works properly.
    ///
    /// 1. For a valid request, the server should return `200 OK`.
    /// 2. For an invalid request according to the extractor, the server should return the error HTTP status code defined by the extractor itself.
    /// 3. For an invalid request according to Valid, the server should return VALIDATION_ERROR_STATUS as the error code.
    ///
    pub trait ValidTest {
        /// The HTTP status code returned when inner extractor failed.
        const ERROR_STATUS_CODE: StatusCode;
        /// The HTTP status code returned when the outer extractor fails.
        /// Use crate::VALIDATION_ERROR_STATUS by default.
        const INVALID_STATUS_CODE: StatusCode = crate::VALIDATION_ERROR_STATUS;
        /// If the response body can be serialized into JSON format
        const JSON_SERIALIZABLE: bool = true;
        /// Build a valid request, the server should return `200 OK`.
        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder;
        /// Build an invalid request according to the extractor, the server should return `Self::ERROR_STATUS_CODE`
        fn set_error_request(builder: RequestBuilder) -> RequestBuilder;
        /// Build an invalid request according to Valid, the server should return VALIDATION_ERROR_STATUS
        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder;
    }

    #[cfg(feature = "extra")]
    pub trait Rejection {
        const STATUS_CODE: StatusCode;
    }
}
