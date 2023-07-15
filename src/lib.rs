#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs, clippy::unwrap_used)]

use axum::extract::rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Form, Json};
use validator::{Validate, ValidationErrors};

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
                (StatusCode::BAD_REQUEST, validate_error.to_string()).into_response()
            }
            ValidRejection::Inner(json_error) => json_error.into_response(),
        }
    }
}

impl From<JsonRejection> for ValidRejection<JsonRejection> {
    fn from(value: JsonRejection) -> Self {
        Self::Inner(value)
    }
}

impl From<QueryRejection> for ValidRejection<QueryRejection> {
    fn from(value: QueryRejection) -> Self {
        Self::Inner(value)
    }
}

impl From<PathRejection> for ValidRejection<PathRejection> {
    fn from(value: PathRejection) -> Self {
        Self::Inner(value)
    }
}

impl From<FormRejection> for ValidRejection<FormRejection> {
    fn from(value: FormRejection) -> Self {
        Self::Inner(value)
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

impl<T: Validate> HasValidate for Json<T> {
    type Validate = T;
    type Rejection = JsonRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> HasValidate for Form<T> {
    type Validate = T;
    type Rejection = FormRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> HasValidate for Query<T> {
    type Validate = T;
    type Rejection = QueryRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> HasValidate for Path<T> {
    type Validate = T;
    type Rejection = PathRejection;
    fn get_validate(&self) -> &T {
        &self.0
    }
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
