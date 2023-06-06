use axum::extract::rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Form, Json};
use validator::{Validate, ValidationErrors};

#[derive(Debug, Clone, Copy, Default)]
pub struct Valid<T>(pub T);

pub enum ValidRejection<E> {
    Valid(ValidationErrors),
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

pub trait Inner0 {
    type Inner: Validate;
    type Rejection;
    fn inner0_ref(&self) -> &Self::Inner;
}

impl<T: Validate> Inner0 for Json<T> {
    type Inner = T;
    type Rejection = JsonRejection;
    fn inner0_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> Inner0 for Form<T> {
    type Inner = T;
    type Rejection = FormRejection;
    fn inner0_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> Inner0 for Query<T> {
    type Inner = T;
    type Rejection = QueryRejection;
    fn inner0_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Validate> Inner0 for Path<T> {
    type Inner = T;
    type Rejection = QueryRejection;
    fn inner0_ref(&self) -> &T {
        &self.0
    }
}

#[async_trait]
impl<S, B, T> FromRequest<S, B> for Valid<T>
where
    S: Send + Sync + 'static,
    B: Send + Sync + 'static,
    T: Inner0 + FromRequest<S, B>,
    T::Inner: Validate,
    <T as Inner0>::Rejection: IntoResponse,
    ValidRejection<<T as Inner0>::Rejection>: From<<T as FromRequest<S, B>>::Rejection>,
{
    type Rejection = ValidRejection<<T as Inner0>::Rejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let valid = T::from_request(req, state).await?;
        valid.inner0_ref().validate()?;
        Ok(Valid(valid))
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for Valid<T>
where
    S: Send + Sync + 'static,
    T: Inner0 + FromRequestParts<S>,
    T::Inner: Validate,
    <T as Inner0>::Rejection: IntoResponse,
    ValidRejection<<T as Inner0>::Rejection>: From<<T as FromRequestParts<S>>::Rejection>,
{
    type Rejection = ValidRejection<<T as Inner0>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let valid = T::from_request_parts(parts, state).await?;
        valid.inner0_ref().validate()?;
        Ok(Valid(valid))
    }
}
