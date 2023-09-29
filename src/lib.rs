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
#[cfg(feature = "typed_multipart")]
pub mod typed_multipart;
#[cfg(feature = "yaml")]
pub mod yaml;

use axum::async_trait;
use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use validator::{Validate, ValidateArgs, ValidationErrors};

/// Http status code returned when there are validation errors.
#[cfg(feature = "422")]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;
/// Http status code returned when there are validation errors.
#[cfg(not(feature = "422"))]
pub const VALIDATION_ERROR_STATUS: StatusCode = StatusCode::BAD_REQUEST;

/// # `Valid` data extractor
///
/// This extractor can be used in combination with axum's extractors like
/// Json, Form, Query, Path, etc to validate their inner data automatically.
/// It can also work with custom extractors that implement the `HasValidate` trait.
///
/// See the docs for each integration module to find examples of using
/// `Valid` with that extractor.
///
/// For examples with custom extractors, check out the `tests/custom.rs` file.
///
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

impl<T: Display, A> Display for ValidEx<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Valid<E> {
    /// Consume the `Valid` extractor and returns the inner type.
    pub fn into_inner(self) -> E {
        self.0
    }
}

/// # `ValidEx` data extractor
///
/// `ValidEx` can be used with extractors from the various modules.
/// Refer to the examples for `Valid` in each module - the usage of
/// `ValidEx` is similar, except the inner data type implements
/// `ValidateArgs` instead of `Validate`.
///
/// `ValidateArgs` is usually automatically implemented by validator's
/// derive macros. Refer to validator's documentation for details.
///
/// Note that the documentation for each module currently only shows  
/// examples of `Valid`, and does not demonstrate concrete usage of
/// `ValidEx`, but the usage is analogous.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidEx<E, A>(pub E, pub A);

impl<E, A> Deref for ValidEx<E, A> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E, A> DerefMut for ValidEx<E, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Valid<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E, A> ValidEx<E, A> {
    /// Consume the `ValidEx` extractor and returns the inner type.
    pub fn into_inner(self) -> E {
        self.0
    }
}

fn response_builder(ve: ValidationErrors) -> Response {
    #[cfg(feature = "into_json")]
    {
        (VALIDATION_ERROR_STATUS, axum::Json(ve)).into_response()
    }
    #[cfg(not(feature = "into_json"))]
    {
        (VALIDATION_ERROR_STATUS, ve.to_string()).into_response()
    }
}

/// `Arguments` provides the validation arguments for the data type `T`.
///
/// This trait has an associated type `T` which represents the data type to
/// validate. `T` must implement the `ValidateArgs` trait which defines the
/// validation logic.
///
pub trait Arguments<'a> {
    /// The data type to validate using this arguments
    type T: ValidateArgs<'a>;
    /// This method gets the arguments required by `ValidateArgs::validate_args`
    fn get(&'a self) -> <<Self as Arguments<'a>>::T as ValidateArgs<'a>>::Args;
}

/// `ValidRejection` is returned when the `Valid` extractor fails.
///
#[derive(Debug)]
pub enum ValidRejection<E> {
    /// Validation errors
    Valid(ValidationErrors),
    /// Inner extractor error
    Inner(E),
}

impl<E: Display> Display for ValidRejection<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidRejection::Valid(errors) => write!(f, "{errors}"),
            ValidRejection::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for ValidRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidRejection::Valid(ve) => Some(ve),
            ValidRejection::Inner(e) => Some(e),
        }
    }
}

impl<E> From<ValidationErrors> for ValidRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

impl<E: IntoResponse> IntoResponse for ValidRejection<E> {
    fn into_response(self) -> Response {
        match self {
            ValidRejection::Valid(ve) => response_builder(ve),
            ValidRejection::Inner(e) => e.into_response(),
        }
    }
}

/// Trait for types that can supply a reference that can be validated.
///
/// Extractor types `T` that implement this trait can be used with `Valid`.
///
pub trait HasValidate {
    /// Inner type that can be validated for correctness
    type Validate: Validate;
    /// Get the inner value
    fn get_validate(&self) -> &Self::Validate;
}

/// Trait for types that can supply a reference that can be validated using arguments.
///
/// Extractor types `T` that implement this trait can be used with `ValidEx`.
///
pub trait HasValidateArgs<'v> {
    /// Inner type that can be validated using arguments
    type ValidateArgs: ValidateArgs<'v>;
    /// Get the inner value
    fn get_validate_args(&self) -> &Self::ValidateArgs;
}

#[async_trait]
impl<State, Body, Extractor> FromRequest<State, Body> for Valid<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Extractor: HasValidate + FromRequest<State, Body>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<State, Extractor> FromRequestParts<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequestParts<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<State, Body, Extractor, Args> FromRequest<State, Body> for ValidEx<Extractor, Args>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Args: Send
        + Sync
        + FromRef<State>
        + for<'a> Arguments<'a, T = <Extractor as HasValidateArgs<'a>>::ValidateArgs>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequest<State, Body>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v>,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;

        inner.get_validate_args().validate_args(arguments.get())?;
        Ok(ValidEx(inner, arguments))
    }
}

#[async_trait]
impl<State, Extractor, Args> FromRequestParts<State> for ValidEx<Extractor, Args>
where
    State: Send + Sync,
    Args: Send
        + Sync
        + FromRef<State>
        + for<'a> Arguments<'a, T = <Extractor as HasValidateArgs<'a>>::ValidateArgs>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequestParts<State>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v>,
{
    type Rejection = ValidRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate_args().validate_args(arguments.get())?;
        Ok(ValidEx(inner, arguments))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Valid, ValidRejection};
    use reqwest::{RequestBuilder, StatusCode};
    use serde::Serialize;
    use std::error::Error;
    use std::io;
    use std::ops::{Deref, DerefMut};
    use validator::{ValidationError, ValidationErrors};

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

    const TEST: &str = "test";

    #[test]
    fn deref_deref_mut_into_inner() {
        let mut inner = String::from(TEST);
        let mut v = Valid(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(TEST);
        v.deref_mut().push_str(TEST);
        assert_eq!(&inner, v.deref());
        assert_eq!(inner, v.into_inner());
    }

    #[test]
    fn display_error() {
        // ValidRejection::Valid Display
        let mut ve = ValidationErrors::new();
        ve.add(TEST, ValidationError::new(TEST));
        let vr = ValidRejection::<String>::Valid(ve.clone());
        assert_eq!(vr.to_string(), ve.to_string());

        // ValidRejection::Inner Display
        let inner = String::from(TEST);
        let vr = ValidRejection::<String>::Inner(inner.clone());
        assert_eq!(inner.to_string(), vr.to_string());

        // ValidRejection::Valid Error
        let mut ve = ValidationErrors::new();
        ve.add(TEST, ValidationError::new(TEST));
        let vr = ValidRejection::<io::Error>::Valid(ve.clone());
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<ValidationErrors>().is_some())
        );

        // ValidRejection::Valid Error
        let vr = ValidRejection::<io::Error>::Inner(io::Error::new(io::ErrorKind::Other, TEST));
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<io::Error>().is_some())
        );
    }
}
