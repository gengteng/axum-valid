//! # Garde support

#[cfg(test)]
pub mod test;

use crate::{HasValidate, VALIDATION_ERROR_STATUS};
use axum::async_trait;
use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use garde::Validate;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

/// # `Garde` data extractor
///
#[derive(Debug, Clone, Copy, Default)]
pub struct Garde<E>(pub E);

impl<E> Deref for Garde<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Garde<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Garde<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Garde<E> {
    /// Consumes the `ValidEx` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

fn response_builder(ve: garde::Report) -> Response {
    #[cfg(feature = "into_json")]
    {
        (VALIDATION_ERROR_STATUS, axum::Json(ve)).into_response()
    }
    #[cfg(not(feature = "into_json"))]
    {
        (VALIDATION_ERROR_STATUS, ve.to_string()).into_response()
    }
}

/// `ValidRejection` is returned when the `Valid` extractor fails.
///
/// This enumeration captures two types of errors that can occur when using `Valid`: errors related to the validation
/// logic itself (encapsulated in `Valid`), and errors that may arise within the inner extractor (represented by `Inner`).
///
#[derive(Debug)]
pub enum GardeRejection<E> {
    /// `Valid` variant captures errors related to the validation logic. It contains `garde::Report`
    /// which is a collection of validation failures for each field.
    Report(garde::Report),
    /// `Inner` variant represents potential errors that might occur within the inner extractor.
    Inner(E),
}

impl<E: Display> Display for GardeRejection<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GardeRejection::Report(errors) => write!(f, "{errors}"),
            GardeRejection::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for GardeRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            GardeRejection::Report(ve) => Some(ve),
            GardeRejection::Inner(e) => Some(e),
        }
    }
}

impl<E> From<garde::Report> for GardeRejection<E> {
    fn from(value: garde::Report) -> Self {
        Self::Report(value)
    }
}

impl<E: IntoResponse> IntoResponse for GardeRejection<E> {
    fn into_response(self) -> Response {
        match self {
            GardeRejection::Report(ve) => response_builder(ve),
            GardeRejection::Inner(e) => e.into_response(),
        }
    }
}

#[async_trait]
impl<State, Body, Extractor, Context> FromRequest<State, Body> for Garde<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Context: Send + Sync + FromRef<State>,
    Extractor: HasValidate + FromRequest<State, Body>,
    <Extractor as HasValidate>::Validate: garde::Validate<Context = Context>,
{
    type Rejection = GardeRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let context: Context = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(GardeRejection::Inner)?;

        inner.get_validate().validate(&context)?;
        Ok(Garde(inner))
    }
}

#[async_trait]
impl<State, Extractor, Context> FromRequestParts<State> for Garde<Extractor>
where
    State: Send + Sync,
    Context: Send + Sync + FromRef<State>,
    Extractor: HasValidate + FromRequestParts<State>,
    <Extractor as HasValidate>::Validate: garde::Validate<Context = Context>,
{
    type Rejection = GardeRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let context: Context = FromRef::from_ref(state);
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(GardeRejection::Inner)?;
        inner.get_validate().validate(&context)?;
        Ok(Garde(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GARDE: &str = "garde";

    #[test]
    fn garde_deref_deref_mut_into_inner() {
        let mut inner = String::from(GARDE);
        let mut v = Garde(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(GARDE);
        v.deref_mut().push_str(GARDE);
        assert_eq!(&inner, v.deref());
        assert_eq!(inner, v.into_inner());
    }
}
