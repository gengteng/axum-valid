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

/// `ValidationContext` configures the response returned when validation fails.
///
/// By providing a ValidationContext to the Valid extractor, you can customize
/// the HTTP status code and response body returned on validation failure.
///
#[derive(Debug, Copy, Clone)]
pub struct ValidationContext<Arguments> {
    /// Validation error response builder
    response_builder: fn(ValidationErrors) -> Response,
    arguments: Arguments,
}

#[cfg(feature = "json")]
fn json_response_builder(ve: ValidationErrors) -> Response {
    {
        (VALIDATION_ERROR_STATUS, axum::Json(ve)).into_response()
    }
}

fn string_response_builder(ve: ValidationErrors) -> Response {
    {
        (VALIDATION_ERROR_STATUS, ve.to_string()).into_response()
    }
}

impl<Arguments: Default> Default for ValidationContext<Arguments> {
    fn default() -> Self {
        fn response_builder(ve: ValidationErrors) -> Response {
            #[cfg(feature = "into_json")]
            {
                json_response_builder(ve)
            }
            #[cfg(not(feature = "into_json"))]
            {
                string_response_builder(ve)
            }
        }

        Self {
            response_builder,
            arguments: Arguments::default(),
        }
    }
}

impl ValidationContext<()> {
    /// Construct a `ValidationContext` with a custom response builder function
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use axum::{response::IntoResponse, Json};
    /// use axum::http::StatusCode;
    /// use axum::response::Response;
    /// use validator::ValidationErrors;
    /// use axum_valid::ValidationContext;
    ///
    ///
    /// fn custom_response(errors: ValidationErrors) -> Response {
    ///   // return response with custom status code and body
    ///   (StatusCode::NOT_FOUND, Json(errors)).into_response()
    /// }
    ///
    /// let context = ValidationContext::custom(custom_response);
    /// ```
    pub fn custom(response_builder: fn(ValidationErrors) -> Response) -> Self {
        Self {
            response_builder,
            arguments: (),
        }
    }

    /// Construct a ValidationContext that returns a string response
    ///
    /// This will return a response with the validation errors formatted as a string
    /// The response status code will be `400 Bad Request` by default, or `422 Unprocessable Entity` if the `422` feature is enabled.
    pub fn string() -> Self {
        Self {
            response_builder: string_response_builder,
            arguments: (),
        }
    }

    /// Construct a ValidationContext that returns a JSON response
    ///
    /// This will return a response with the validation errors serialized as JSON.
    /// The response status code will be `400 Bad Request` by default, or `422 Unprocessable Entity` if the `422` feature is enabled.
    ///
    /// Requires the `json` feature to be enabled.
    #[cfg(feature = "json")]
    pub fn json() -> Self {
        Self {
            response_builder: json_response_builder,
            arguments: (),
        }
    }

    /// Creates a new `ValidationContext` with arguments.
    pub fn with_arguments<Arguments>(self, arguments: Arguments) -> ValidationContext<Arguments> {
        ValidationContext {
            response_builder: self.response_builder,
            arguments,
        }
    }
}

impl<Arguments> ValidationContext<Arguments> {
    /// Construct a `ValidationContext` with a custom response builder function
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use axum::{response::IntoResponse, Json};
    /// use axum::http::StatusCode;
    /// use axum::response::Response;
    /// use validator::ValidationErrors;
    /// use axum_valid::ValidationContext;
    ///
    /// struct MyArguments;
    ///
    /// fn custom_response(errors: ValidationErrors) -> Response {
    ///   // return response with custom status code and body
    ///   (StatusCode::NOT_FOUND, Json(errors)).into_response()
    /// }
    ///
    /// let context = ValidationContext::custom_with_arguments(custom_response, MyArguments);
    /// ```
    pub fn custom_with_arguments(
        response_builder: fn(ValidationErrors) -> Response,
        arguments: Arguments,
    ) -> Self {
        Self {
            response_builder,
            arguments,
        }
    }

    /// Construct a ValidationContext that returns a string response
    ///
    /// This will return a response with the validation errors formatted as a string
    /// The response status code will be `400 Bad Request` by default, or `422 Unprocessable Entity` if the `422` feature is enabled.
    pub fn string_with_arguments(arguments: Arguments) -> Self {
        Self {
            response_builder: string_response_builder,
            arguments,
        }
    }

    /// Construct a ValidationContext that returns a JSON response
    ///
    /// This will return a response with the validation errors serialized as JSON.
    /// The response status code will be `400 Bad Request` by default, or `422 Unprocessable Entity` if the `422` feature is enabled.
    ///
    /// Requires the `json` feature to be enabled.
    #[cfg(feature = "json")]
    pub fn json_with_arguments(arguments: Arguments) -> Self {
        Self {
            response_builder: json_response_builder,
            arguments,
        }
    }

    /// Creates a new `ValidationContext` with empty arguments.
    pub fn without_arguments(&self) -> ValidationContext<()> {
        ValidationContext {
            response_builder: self.response_builder,
            arguments: (),
        }
    }
}

impl FromRef<()> for ValidationContext<()> {
    fn from_ref(_: &()) -> Self {
        ValidationContext::default()
    }
}

/// `ValidError` is the error type returned when the `Valid` extractor fails.
///
/// It has two variants:
///
/// - Valid: Contains validation errors (ValidationErrors) when validation fails.
/// - Inner: Contains the inner extractor error when the internal extractor fails.
///
#[derive(Debug)]
pub enum ValidError<E> {
    /// Validation errors
    Valid(ValidationErrors),
    /// Inner extractor error
    Inner(E),
}

impl<E: Display> Display for ValidError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidError::Valid(errors) => write!(f, "{errors}"),
            ValidError::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for ValidError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidError::Valid(ve) => Some(ve),
            ValidError::Inner(e) => Some(e),
        }
    }
}

impl<E> From<ValidationErrors> for ValidError<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

/// `ValidRejection` is returned when the `Valid` extractor fails.
///
/// It contains the underlying `ValidError` and handles converting it
/// into a proper HTTP response.
pub struct ValidRejection<E> {
    error: ValidError<E>,
    response_builder: fn(ValidationErrors) -> Response,
}

impl<E: Display> Display for ValidRejection<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.error {
            ValidError::Valid(errors) => write!(f, "{errors}"),
            ValidError::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Debug> Debug for ValidRejection<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.error, f)
    }
}

impl<E: Error + 'static> Error for ValidRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.error {
            ValidError::Valid(ve) => Some(ve),
            ValidError::Inner(e) => Some(e),
        }
    }
}

impl<E: IntoResponse> IntoResponse for ValidRejection<E> {
    fn into_response(self) -> Response {
        match self.error {
            ValidError::Valid(ve) => (self.response_builder)(ve),
            ValidError::Inner(e) => e.into_response(),
        }
    }
}

/// Trait for types that can provide a reference that can be validated for correctness.
///
/// Extractor types `T` that implement this trait can be used with `Valid`.
///
pub trait HasValidate {
    /// Inner type that can be validated for correctness
    type Validate: Validate;
    /// Get the inner value
    fn get_validate(&self) -> &Self::Validate;
}

///
pub trait HasValidateArgs<'v> {
    /// Inner type that can be validated for correctness
    type ValidateArgs: ValidateArgs<'v>;
    /// Get the inner value
    fn get_validate_args(&self) -> &Self::ValidateArgs;
}

#[async_trait]
impl<S, B, E> FromRequest<S, B> for Valid<E>
where
    S: Send + Sync,
    B: Send + Sync + 'static,
    E: HasValidate + FromRequest<S, B>,
    E::Validate: Validate,
    ValidationContext<()>: FromRef<S>,
{
    type Rejection = ValidRejection<<E as FromRequest<S, B>>::Rejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let context: ValidationContext<()> = FromRef::from_ref(state);
        let inner = E::from_request(req, state)
            .await
            .map_err(|e| ValidRejection {
                error: ValidError::Inner(e),
                response_builder: context.response_builder,
            })?;
        inner
            .get_validate()
            .validate()
            .map_err(|e| ValidRejection {
                error: ValidError::Valid(e),
                response_builder: context.response_builder,
            })?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<S, E> FromRequestParts<S> for Valid<E>
where
    S: Send + Sync,
    E: HasValidate + FromRequestParts<S>,
    E::Validate: Validate,
    ValidationContext<()>: FromRef<S>,
{
    type Rejection = ValidRejection<<E as FromRequestParts<S>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let context: ValidationContext<()> = FromRef::from_ref(state);
        let inner = E::from_request_parts(parts, state)
            .await
            .map_err(|e| ValidRejection {
                error: ValidError::Inner(e),
                response_builder: context.response_builder,
            })?;
        inner
            .get_validate()
            .validate()
            .map_err(|e| ValidRejection {
                error: ValidError::Valid(e),
                response_builder: context.response_builder,
            })?;
        Ok(Valid(inner))
    }
}

#[async_trait]
impl<S, B, E, Arguments> FromRequest<S, B> for ValidEx<E, Arguments>
where
    S: Send + Sync,
    B: Send + Sync + 'static,
    Arguments: Send + Sync,
    E: for<'v> HasValidateArgs<'v> + FromRequest<S, B>,
    for<'v> <E as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v, Args = &'v Arguments>,
    ValidationContext<Arguments>: FromRef<S>,
{
    type Rejection = ValidRejection<<E as FromRequest<S, B>>::Rejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let ValidationContext {
            response_builder,
            arguments,
        }: ValidationContext<Arguments> = FromRef::from_ref(state);
        let inner = E::from_request(req, state)
            .await
            .map_err(|e| ValidRejection {
                error: ValidError::Inner(e),
                response_builder,
            })?;

        inner
            .get_validate_args()
            .validate_args(&arguments)
            .map_err(|e| ValidRejection {
                error: ValidError::Valid(e),
                response_builder,
            })?;
        Ok(ValidEx(inner, arguments))
    }
}

#[async_trait]
impl<S, E, Arguments> FromRequestParts<S> for ValidEx<E, Arguments>
where
    S: Send + Sync,
    Arguments: Send + Sync,
    E: for<'v> HasValidateArgs<'v> + FromRequestParts<S>,
    for<'v> <E as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v, Args = &'v Arguments>,
    ValidationContext<Arguments>: FromRef<S>,
{
    type Rejection = ValidRejection<<E as FromRequestParts<S>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ValidationContext {
            response_builder,
            arguments,
        }: ValidationContext<Arguments> = FromRef::from_ref(state);
        let inner = E::from_request_parts(parts, state)
            .await
            .map_err(|e| ValidRejection {
                error: ValidError::Inner(e),
                response_builder,
            })?;
        inner
            .get_validate_args()
            .validate_args(&arguments)
            .map_err(|e| ValidRejection {
                error: ValidError::Valid(e),
                response_builder,
            })?;

        Ok(ValidEx(inner, arguments))
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{Valid, ValidError};
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
        let vr = ValidError::<String>::Valid(ve.clone());
        assert_eq!(vr.to_string(), ve.to_string());

        // ValidRejection::Inner Display
        let inner = String::from(TEST);
        let vr = ValidError::<String>::Inner(inner.clone());
        assert_eq!(inner.to_string(), vr.to_string());

        // ValidRejection::Valid Error
        let mut ve = ValidationErrors::new();
        ve.add(TEST, ValidationError::new(TEST));
        let vr = ValidError::<io::Error>::Valid(ve.clone());
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<ValidationErrors>().is_some())
        );

        // ValidRejection::Valid Error
        let vr = ValidError::<io::Error>::Inner(io::Error::new(io::ErrorKind::Other, TEST));
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<io::Error>().is_some())
        );
    }
}
