//! # Garde support
//!
//! ## Feature
//!
//! Enable the `garde` feature to use `Garde<E>`.
//!

#[cfg(test)]
pub mod test;

use crate::{HasValidate, ValidationRejection};
use axum::extract::{FromRef, FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use garde::{Report, Validate};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

/// # `Garde` data extractor
///
/// Garde uses garde to validate data, supporting validation with or without arguments.
///
/// If not using arguments, its usage is similar to `Valid`. However, if your axum router uses a state, you need to implement `FromRef<StateType>` for `()`.
///
/// If using arguments, you must pass the arguments to Garde extractor via state, meaning implementing `FromRef<StateType>` for your validation arguments type.
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
    /// Consumes the `Garde` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for Garde<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// `GardeRejection` is returned when the `Garde` extractor fails.
///
pub type GardeRejection<E> = ValidationRejection<Report, E>;

impl<E> From<Report> for GardeRejection<E> {
    fn from(value: Report) -> Self {
        Self::Valid(value)
    }
}

impl<State, Extractor, Context> FromRequest<State> for Garde<Extractor>
where
    State: Send + Sync,
    Context: Send + Sync + FromRef<State>,
    Extractor: HasValidate + FromRequest<State>,
    <Extractor as HasValidate>::Validate: Validate<Context = Context>,
{
    type Rejection = GardeRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let context: Context = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(GardeRejection::Inner)?;

        inner.get_validate().validate_with(&context)?;
        Ok(Garde(inner))
    }
}

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
        inner.get_validate().validate_with(&context)?;
        Ok(Garde(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use garde::{Path, Report};
    use std::error::Error;
    use std::io;

    const GARDE: &str = "garde";

    #[test]
    fn garde_deref_deref_mut_into_inner() {
        let mut inner = String::from(GARDE);
        let mut v = Garde(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(GARDE);
        v.deref_mut().push_str(GARDE);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());
    }

    #[test]
    fn display_error() {
        // GardeRejection::Valid Display
        let mut report = Report::new();
        report.append(Path::empty(), garde::Error::new(GARDE));
        let s = report.to_string();
        let vr = GardeRejection::<String>::Valid(report);
        assert_eq!(vr.to_string(), s);

        // GardeRejection::Inner Display
        let inner = String::from(GARDE);
        let vr = GardeRejection::<String>::Inner(inner.clone());
        assert_eq!(inner.to_string(), vr.to_string());

        // GardeRejection::Valid Error
        let mut report = Report::new();
        report.append(Path::empty(), garde::Error::new(GARDE));
        let vr = GardeRejection::<io::Error>::Valid(report);
        assert!(matches!(vr.source(), Some(source) if source.downcast_ref::<Report>().is_some()));

        // GardeRejection::Valid Error
        let vr = GardeRejection::<io::Error>::Inner(io::Error::new(io::ErrorKind::Other, GARDE));
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<io::Error>().is_some())
        );
    }
}
