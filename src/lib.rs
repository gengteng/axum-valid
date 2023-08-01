#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs, clippy::unwrap_used)]

#[cfg(feature = "form")]
pub mod form;
#[cfg(feature = "json")]
pub mod json;
pub mod path;
#[cfg(feature = "query")]
pub mod query;

use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use validator::{Validate, ValidationErrors};

/// Http status code returned when there are validation errors.
#[cfg(feature = "422")]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;
/// Http status code returned when there are validation errors.
#[cfg(not(feature = "422"))]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::BAD_REQUEST;

/// Valid entity extractor
#[derive(Debug, Clone, Copy, Default)]
pub struct Valid<T>(pub T);

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
    /// If the inner extractor fails it'll use this "rejection" type.
    /// A rejection is a kind of error that can be converted into a response.
    type Rejection: IntoResponse;
    /// get the inner type
    fn get_validate(&self) -> &Self::Validate;
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Valid<T>
where
    S: Send + Sync + 'static,
    B: Send + Sync + 'static,
    T: HasValidate + FromRequest<S, B>,
    T::Validate: Validate,
    ValidRejection<<T as HasValidate>::Rejection>: From<<T as FromRequest<S, B>>::Rejection>,
{
    type Rejection = ValidRejection<<T as HasValidate>::Rejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let inner = T::from_request(req, state).await?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for Valid<T>
where
    S: Send + Sync + 'static,
    T: HasValidate + FromRequestParts<S>,
    T::Validate: Validate,
    ValidRejection<<T as HasValidate>::Rejection>: From<<T as FromRequestParts<S>>::Rejection>,
{
    type Rejection = ValidRejection<<T as HasValidate>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let inner = T::from_request_parts(parts, state).await?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}
