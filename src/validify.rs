//! # Validify support
//!
//! ## Feature
//!
//! Enable the `validify` feature to use `Validated<E>`, `Modified<E>`, `Validified<E>` and `ValidifiedByRef<E>`.
//!

#[cfg(test)]
pub mod test;

use crate::{HasValidate, ValidationRejection};
use axum::extract::{FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use validify::{Modify, Validate, ValidationErrors, ValidifyPayload};

/// # `Validated` data extractor
///
/// `Validated` provides simple data validation based on `validify`.
///
/// It only does validation, usage is similar to `Valid`.
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
    /// Consumes the `Validated` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for Validated<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// # `Modified` data extractor / response
///
/// ## Extractor
///
/// `Modified` uses `validify`'s modification capabilities to alter data, without validation.
///
/// Operations like trimming and case modification can be done based on `modify` attributes.
///
/// ## Response
///
/// `Modified` also implements the `IntoResponse` trait. When its inner `IntoResponse` type also implements the `HasModify` trait:
///
/// `Modified` will call `validify`'s modify method to alter the inner data.
/// Then call the inner type's own `into_response` method to convert it into a HTTP response.
///
/// This allows applying modifications during response conversion by leveraging validify.
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

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for Modified<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationOutput for Modified<T>
where
    T: aide::OperationOutput,
{
    type Inner = T::Inner;

    fn operation_response(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Option<aide::openapi::Response> {
        T::operation_response(ctx, operation)
    }

    fn inferred_responses(
        ctx: &mut aide::gen::GenContext,
        operation: &mut aide::openapi::Operation,
    ) -> Vec<(Option<u16>, aide::openapi::Response)> {
        T::inferred_responses(ctx, operation)
    }
}

/// # `Validified` data extractor
///
/// `Validified` provides construction, modification and validation abilities based on `validify`.
///
/// It requires a serde-based inner extractor.
///
/// And can treat missing fields as validation errors.
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
    /// Consumes the `Validified` and returns the modified and validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for Validified<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// # `ValidifiedByRef` data extractor
///
/// `ValidifiedByRef` is similar to `Validified`, but operates via reference.
///
/// Suitable for inner extractors not based on `serde`.
///
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidifiedByRef<E>(pub E);

impl<E> Deref for ValidifiedByRef<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for ValidifiedByRef<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for ValidifiedByRef<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> ValidifiedByRef<E> {
    /// Consumes the `ValidifiedByRef` and returns the modified and validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for ValidifiedByRef<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// `ValidifyRejection` is returned when the `Validated` / `Modified` / `Validified` / `ValidifiedByRef` extractor fails.
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

/// Trait for extractors whose inner data type that can be constructed using some payload,  
/// then modified and validated using `validify`.
///
/// Extractor types `T` that implement this trait can be used with `Validified`.
///
pub trait HasValidify: Sized {
    /// Inner type that can be modified and validated using `validify`.
    type Validify: ValidifyPayload;

    /// Extracts payload from the request,
    /// which will be used to construct the `Self::Validify` type  
    /// and perform modification and validation on it.
    type PayloadExtractor: PayloadExtractor<Payload = <Self::Validify as ValidifyPayload>::Payload>;

    /// Re-packages the validified data back into the inner Extractor type.  
    fn from_validify(v: Self::Validify) -> Self;
}

impl<State, Extractor> FromRequest<State> for Validated<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequest<State>,
    Extractor::Validate: validify::Validate,
{
    type Rejection = ValidifyRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Validated(inner))
    }
}

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

impl<State, Extractor> FromRequest<State> for Modified<Extractor>
where
    State: Send + Sync,
    Extractor: HasModify + FromRequest<State>,
{
    type Rejection = <Extractor as FromRequest<State>>::Rejection;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let mut inner = Extractor::from_request(req, state).await?;
        inner.get_modify().modify();
        Ok(Modified(inner))
    }
}

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

impl<State, Extractor> FromRequest<State> for Validified<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidify,
    Extractor::PayloadExtractor: FromRequest<State>,
{
    type Rejection =
        ValidifyRejection<<Extractor::PayloadExtractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let payload = Extractor::PayloadExtractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?
            .get_payload();
        let validify = Extractor::Validify::validify_from(payload)?;
        Ok(Validified(Extractor::from_validify(validify)))
    }
}

impl<State, Extractor> FromRequestParts<State> for Validified<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidify,
    Extractor::PayloadExtractor: FromRequestParts<State>,
{
    type Rejection =
        ValidifyRejection<<Extractor::PayloadExtractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let payload = Extractor::PayloadExtractor::from_request_parts(parts, state)
            .await
            .map_err(ValidifyRejection::Inner)?
            .get_payload();
        let validify = Extractor::Validify::validify_from(payload)?;
        Ok(Validified(Extractor::from_validify(validify)))
    }
}

impl<State, Extractor> FromRequest<State> for ValidifiedByRef<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + HasModify + FromRequest<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidifyRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let mut inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        inner.get_modify().modify();
        inner.get_validate().validate()?;
        Ok(ValidifiedByRef(inner))
    }
}

impl<State, Extractor> FromRequestParts<State> for ValidifiedByRef<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + HasModify + FromRequestParts<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidifyRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let mut inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidifyRejection::Inner)?;
        inner.get_modify().modify();
        inner.get_validate().validate()?;
        Ok(ValidifiedByRef(inner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Json;
    use serde::Serialize;
    use std::error::Error;
    use std::io;

    const VALIDIFY: &str = "validify";

    #[test]
    fn validify_deref_deref_mut_into_inner() {
        let mut inner = String::from(VALIDIFY);
        let mut v = Validated(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(VALIDIFY);
        v.deref_mut().push_str(VALIDIFY);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());

        let mut inner = String::from(VALIDIFY);
        let mut v = Modified(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(VALIDIFY);
        v.deref_mut().push_str(VALIDIFY);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());

        let mut inner = String::from(VALIDIFY);
        let mut v = Validified(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(VALIDIFY);
        v.deref_mut().push_str(VALIDIFY);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());

        let mut inner = String::from(VALIDIFY);
        let mut v = ValidifiedByRef(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(VALIDIFY);
        v.deref_mut().push_str(VALIDIFY);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());
    }

    #[test]
    fn display_error() {
        // ValidifyRejection::Valid Display
        let mut report = ValidationErrors::new();
        report.add(validify::ValidationError::new_schema(VALIDIFY));
        let s = report.to_string();
        let vr = ValidifyRejection::<String>::Valid(report);
        assert_eq!(vr.to_string(), s);

        // ValidifyRejection::Inner Display
        let inner = String::from(VALIDIFY);
        let vr = ValidifyRejection::<String>::Inner(inner.clone());
        assert_eq!(inner.to_string(), vr.to_string());

        // ValidifyRejection::Valid Error
        let mut report = ValidationErrors::new();
        report.add(validify::ValidationError::new_schema(VALIDIFY));
        let vr = ValidifyRejection::<io::Error>::Valid(report);
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<ValidationErrors>().is_some())
        );

        // ValidifyRejection::Valid Error
        let vr =
            ValidifyRejection::<io::Error>::Inner(io::Error::new(io::ErrorKind::Other, VALIDIFY));
        assert!(
            matches!(vr.source(), Some(source) if source.downcast_ref::<io::Error>().is_some())
        );
    }

    #[test]
    fn modified_into_response() {
        use validify::Validify;
        #[derive(Validify, Serialize)]
        struct Data {
            #[modify(trim)]
            v: String,
        }
        println!(
            "{:?}",
            Modified(Json(Data { v: "a  ".into() })).into_response()
        );
    }
}
