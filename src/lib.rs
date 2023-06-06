use axum::body::HttpBody;
use axum::extract::rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Path, Query};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{async_trait, BoxError, Form, Json};
use serde::de::DeserializeOwned;
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

#[async_trait]
impl<T, S, B> FromRequest<S, B> for Valid<Json<T>>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = ValidRejection<JsonRejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let json = Json::<T>::from_request(req, state).await?;
        json.0.validate()?;
        Ok(Valid(json))
    }
}

impl From<QueryRejection> for ValidRejection<QueryRejection> {
    fn from(value: QueryRejection) -> Self {
        Self::Inner(value)
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for Valid<Query<T>>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ValidRejection<QueryRejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let query = Query::<T>::from_request_parts(parts, state).await?;
        query.validate()?;
        Ok(Valid(query))
    }
}

impl From<PathRejection> for ValidRejection<PathRejection> {
    fn from(value: PathRejection) -> Self {
        Self::Inner(value)
    }
}

#[async_trait]
impl<T, S> FromRequestParts<S> for Valid<Path<T>>
where
    T: DeserializeOwned + Validate + Send,
    S: Send + Sync,
{
    type Rejection = ValidRejection<PathRejection>;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let path = Path::<T>::from_request_parts(parts, state).await?;
        path.validate()?;
        Ok(Valid(path))
    }
}

impl From<FormRejection> for ValidRejection<FormRejection> {
    fn from(value: FormRejection) -> Self {
        Self::Inner(value)
    }
}

#[async_trait]
impl<T, S, B> FromRequest<S, B> for Valid<Form<T>>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
    S: Send + Sync,
{
    type Rejection = ValidRejection<FormRejection>;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let form = Form::<T>::from_request(req, state).await?;
        form.validate()?;
        Ok(Valid(form))
    }
}
