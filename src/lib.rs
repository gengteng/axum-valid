#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs, clippy::unwrap_used)]

#[cfg(feature = "extra")]
pub mod extra;
#[cfg(feature = "form")]
pub mod form;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "msgpack")]
pub mod msgpack;
pub mod path;
#[cfg(feature = "query")]
pub mod query;
#[cfg(test)]
pub mod test;
#[cfg(feature = "typed_header")]
pub mod typed_header;
#[cfg(feature = "yaml")]
pub mod yaml;

use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use std::ops::{Deref, DerefMut};
use validator::{Validate, ValidationErrors};

/// Http status code returned when there are validation errors.
#[cfg(feature = "422")]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;
/// Http status code returned when there are validation errors.
#[cfg(not(feature = "422"))]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::BAD_REQUEST;

/// Valid entity extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct Valid<E>(pub E);

impl<E> Deref for Valid<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Valid<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// If the valid extractor fails it'll use this "rejection" type.
/// This rejection type can be converted into a response.
pub enum ValidRejection<E> {
    /// Validation errors
    Valid(ValidationErrors),
    /// Inner extractor error
    Inner(E),
}

impl<E> From<ValidationErrors> for ValidRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

impl<E: IntoResponse> IntoResponse for ValidRejection<E> {
    fn into_response(self) -> Response {
        match self {
            ValidRejection::Valid(validate_error) => {
                #[cfg(feature = "into_json")]
                {
                    (VALIDATION_ERROR_STATUS, axum::Json(validate_error)).into_response()
                }
                #[cfg(not(feature = "into_json"))]
                {
                    (VALIDATION_ERROR_STATUS, validate_error.to_string()).into_response()
                }
            }
            ValidRejection::Inner(json_error) => json_error.into_response(),
        }
    }
}

/// Trait for types that can provide a reference that can be validated for correctness.
pub trait HasValidate {
    /// Inner type that can be validated for correctness
    type Validate: Validate;
    /// Get the inner value
    fn get_validate(&self) -> &Self::Validate;
}

#[async_trait]
impl<S, B, E> FromRequest<S, B> for Valid<E>
where
    S: Send + Sync + 'static,
    B: Send + Sync + 'static,
    E: HasValidate + FromRequest<S, B>,
    E::Validate: Validate,
{
    type Rejection = ValidRejection<<E as FromRequest<S, B>>::Rejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let inner = E::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<S, E> FromRequestParts<S> for Valid<E>
where
    S: Send + Sync + 'static,
    E: HasValidate + FromRequestParts<S>,
    E::Validate: Validate,
{
    type Rejection = ValidRejection<<E as FromRequestParts<S>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = E::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[cfg(test)]
pub mod tests {
    use reqwest::{RequestBuilder, StatusCode};
    use serde::Serialize;

    /// # Valid test parameter
    pub trait ValidTestParameter: Serialize + 'static {
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
        /// Http status code when inner extractor failed
        const ERROR_STATUS_CODE: StatusCode;
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
