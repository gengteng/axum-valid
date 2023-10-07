//! # Garde support

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
pub struct Garde<E, A>(pub E, pub A);

impl<E, A> Deref for Garde<E, A> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E, A> DerefMut for Garde<E, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display, A> Display for Garde<T, A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E, A> Garde<E, A> {
    /// Consumes the `ValidEx` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }

    /// Returns a reference to the validation arguments.
    ///
    /// This provides access to the `A` type which contains the arguments used
    /// to validate the data. These arguments were passed to the validation
    /// function.
    pub fn arguments<'a>(&'a self) -> <<A as GardeArgument>::T as Validate>::Context
    where
        A: GardeArgument,
    {
        self.1.get()
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

/// `Arguments` provides the validation arguments for the data type `T`.
///
/// This trait has an associated type `T` which represents the data type to
/// validate. `T` must implement the `ValidateArgs` trait which defines the
/// validation logic.
///
/// It's important to mention that types implementing `Arguments` should be a part of the router's state
/// (either through implementing `FromRef<StateType>` or by directly becoming the state)
/// to enable automatic arguments retrieval during validation.
///
pub trait GardeArgument {
    /// The data type to validate using this arguments
    type T: Validate;
    /// This method gets the arguments required by `ValidateArgs::validate_args`
    fn get(&self) -> <<Self as GardeArgument>::T as Validate>::Context;
}

/// `ValidRejection` is returned when the `Valid` extractor fails.
///
/// This enumeration captures two types of errors that can occur when using `Valid`: errors related to the validation
/// logic itself (encapsulated in `Valid`), and errors that may arise within the inner extractor (represented by `Inner`).
///
#[derive(Debug)]
pub enum GardeRejection<E> {
    /// `Valid` variant captures errors related to the validation logic. It contains `ValidationErrors`
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
impl<State, Body, Extractor, Args> FromRequest<State, Body> for Garde<Extractor, Args>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Args: Send + Sync + FromRef<State> + GardeArgument<T = <Extractor as HasValidate>::Validate>,
    Extractor: HasValidate + FromRequest<State, Body>,
    <Extractor as HasValidate>::Validate: garde::Validate,
{
    type Rejection = GardeRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(GardeRejection::Inner)?;

        inner.get_validate().validate(&arguments.get())?;
        Ok(Garde(inner, arguments))
    }
}

#[async_trait]
impl<State, Extractor, Args> FromRequestParts<State> for Garde<Extractor, Args>
where
    State: Send + Sync,
    Args: Send + Sync + FromRef<State> + GardeArgument<T = <Extractor as HasValidate>::Validate>,
    Extractor: HasValidate + FromRequestParts<State>,
    <Extractor as HasValidate>::Validate: garde::Validate,
{
    type Rejection = GardeRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(GardeRejection::Inner)?;
        inner.get_validate().validate(&arguments.get())?;
        Ok(Garde(inner, arguments))
    }
}
