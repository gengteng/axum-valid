//! # Validify support
//!
//! ## Feature
//!
//! Enable the `validify` feature to use `Validated<E>`, `Modified<E>` and `Validified<E>`.
//!

#[cfg(test)]
pub mod test;

use crate::{HasValidate, ValidationRejection};
use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use validify::{Modify, Validate, ValidationErrors, Validify};

/// # `Validated` data extractor
///
#[derive(Debug, Clone, Copy, Default)]
pub struct Validated<E>(pub E);

impl<E> Deref for Validated<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Validated<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Validated<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Validated<E> {
    /// Consumes the `Validified` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

/// # `Modified` data extractor / response
///
#[derive(Debug, Clone, Copy, Default)]
pub struct Modified<E>(pub E);

impl<E> Deref for Modified<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Modified<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Modified<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Modified<E> {
    /// Consumes the `Modified` and returns the modified data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// modified.
    pub fn into_inner(self) -> E {
        self.0
    }
}

impl<E: IntoResponse + HasModify> IntoResponse for Modified<E> {
    fn into_response(mut self) -> Response {
        self.get_modify().modify();
        self.0.into_response()
    }
}

/// # `Validified` data extractor
///
#[derive(Debug, Clone, Copy, Default)]
pub struct Validified<E>(pub E);

impl<E> Deref for Validified<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for Validified<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for Validified<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Validified<E> {
    /// Consumes the `ValidifiedMut` and returns the modified and validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

/// `ValidifyRejection` is returned when the `Validified` / `ValidifiedMut` / `Modified` extractor fails.
///
pub type ValidifyRejection<E> = ValidationRejection<ValidationErrors, E>;

impl<E> From<ValidationErrors> for ValidifyRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

/// Trait for types that can supply a reference that can be modified.
///
/// Extractor types `T` that implement this trait can be used with `Modified`.
///
pub trait HasModify {
    /// Inner type that can be modified
    type Modify: Modify;
    /// Get the inner value
    fn get_modify(&mut self) -> &mut Self::Modify;
}

/// Extractor to extract payload for constructing data
pub trait PayloadExtractor {
    /// Type of payload for constructing data
    type Payload;
    /// Get payload from the extractor
    fn get_payload(self) -> Self::Payload;
}

/// Trait for types that can supply a reference that can be modified and validated using `validify`.
///
/// Extractor types `T` that implement this trait can be used with `Validified`.
///
pub trait HasValidify: Sized {
    /// Inner type that can be modified and validated using `validify`.
    type Validify: Validify;
    ///
    type PayloadExtractor: PayloadExtractor<Payload = <Self::Validify as Validify>::Payload>;
    ///
    fn from_validified(v: Self::Validify) -> Self;
}

#[async_trait]
impl<State, Body, Extractor> FromRequest<State, Body> for Validated<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Extractor: HasValidate + FromRequest<State, Body>,
    Extractor::Validate: validify::Validate,
{
    type Rejection = ValidifyRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Validated(inner))
    }
}

#[async_trait]
impl<State, Extractor> FromRequestParts<State> for Validated<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequestParts<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidifyRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Validated(inner))
    }
}

#[async_trait]
impl<State, Body, Extractor> FromRequest<State, Body> for Modified<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Extractor: HasModify + FromRequest<State, Body>,
{
    type Rejection = <Extractor as FromRequest<State, Body>>::Rejection;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let mut inner = Extractor::from_request(req, state).await?;
        inner.get_modify().modify();
        Ok(Modified(inner))
    }
}

#[async_trait]
impl<State, Extractor> FromRequestParts<State> for Modified<Extractor>
where
    State: Send + Sync,
    Extractor: HasModify + FromRequestParts<State>,
{
    type Rejection = <Extractor as FromRequestParts<State>>::Rejection;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let mut inner = Extractor::from_request_parts(parts, state).await?;
        inner.get_modify().modify();
        Ok(Modified(inner))
    }
}

#[async_trait]
impl<State, Body, Extractor> FromRequest<State, Body> for Validified<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Extractor: HasValidify,
    Extractor::Validify: Validify,
    Extractor::PayloadExtractor: FromRequest<State, Body>,
{
    type Rejection =
        ValidifyRejection<<Extractor::PayloadExtractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let payload = Extractor::PayloadExtractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        Ok(Validified(Extractor::from_validified(
            Extractor::Validify::validify(payload.get_payload())?,
        )))
    }
}

#[async_trait]
impl<State, Extractor> FromRequestParts<State> for Validified<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidify,
    Extractor::Validify: Validify,
    Extractor::PayloadExtractor: FromRequestParts<State>,
{
    type Rejection =
        ValidifyRejection<<Extractor::PayloadExtractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let payload = Extractor::PayloadExtractor::from_request_parts(parts, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        Ok(Validified(Extractor::from_validified(
            Extractor::Validify::validify(payload.get_payload())?,
        )))
    }
}
