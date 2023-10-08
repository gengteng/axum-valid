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
#[cfg(test)]
#[cfg(all(feature = "garde", feature = "validator"))]
pub mod test;
#[cfg(feature = "typed_header")]
pub mod typed_header;
#[cfg(feature = "typed_multipart")]
pub mod typed_multipart;
#[cfg(feature = "validator")]
pub mod validator;
#[cfg(feature = "yaml")]
pub mod yaml;

use axum::http::StatusCode;

/// Http status code returned when there are validation errors.
#[cfg(feature = "422")]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;
/// Http status code returned when there are validation errors.
#[cfg(not(feature = "422"))]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::BAD_REQUEST;

/// Trait for types that can supply a reference that can be validated.
///
/// Extractor types `T` that implement this trait can be used with `Valid` or `Garde`.
///
pub trait HasValidate {
    /// Inner type that can be validated for correctness
    type Validate;
    /// Get the inner value
    fn get_validate(&self) -> &Self::Validate;
}

#[cfg(feature = "validator")]
pub use crate::validator::{Arguments, HasValidateArgs, Valid, ValidEx, ValidRejection};

#[cfg(feature = "garde")]
pub use crate::garde::{Garde, GardeRejection};

#[cfg(test)]
mod tests {
    use reqwest::{RequestBuilder, StatusCode};

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
