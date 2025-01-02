//! # Validator support
//!
//! ## Feature
//!
//! Enable the `validator` feature (enabled by default) to use `Valid<E>` and `ValidEx<E, A>`.
//!

#[cfg(test)]
pub mod test;

use crate::{HasValidate, ValidationRejection};
use axum::extract::{FromRef, FromRequest, FromRequestParts, Request};
use axum::http::request::Parts;
use std::fmt::Display;
use std::ops::{Deref, DerefMut};
use validator::{Validate, ValidateArgs, ValidationErrors};

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

impl<T: Display> Display for Valid<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> Valid<E> {
    /// Consume the `Valid` extractor and returns the inner type.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for Valid<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// # `ValidEx` data extractor
///
/// `ValidEx` can be incorporated with extractors from various modules, similar to `Valid`.
/// Two differences exist between `ValidEx` and `Valid`:
///
/// - The inner data type in `ValidEx` implements `ValidateArgs` instead of `Validate`.
/// - `ValidEx` includes a second field that represents arguments used during validation of the first field.
///
/// The implementation of `ValidateArgs` is often automatically handled by validator's derive macros
/// (for more details, please refer to the validator's documentation).
///
/// Although current module documentation predominantly showcases `Valid` examples, the usage of `ValidEx` is analogous.
///
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidEx<E>(pub E);

impl<E> Deref for ValidEx<E> {
    type Target = E;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<E> DerefMut for ValidEx<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Display> Display for ValidEx<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E> ValidEx<E> {
    /// Consumes the `ValidEx` and returns the validated data within.
    ///
    /// This returns the `E` type which represents the data that has been
    /// successfully validated.
    pub fn into_inner(self) -> E {
        self.0
    }
}

#[cfg(feature = "aide")]
impl<T> aide::OperationInput for ValidEx<T>
where
    T: aide::OperationInput,
{
    fn operation_input(ctx: &mut aide::gen::GenContext, operation: &mut aide::openapi::Operation) {
        T::operation_input(ctx, operation);
    }
}

/// `ValidRejection` is returned when the `Valid` or `ValidEx` extractor fails.
///
pub type ValidRejection<E> = ValidationRejection<ValidationErrors, E>;

impl<E> From<ValidationErrors> for ValidRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
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

impl<State, Extractor> FromRequest<State> for Valid<Extractor>
where
    State: Send + Sync,
    Extractor: HasValidate + FromRequest<State>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

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

impl<State, Extractor, Args> FromRequest<State> for ValidEx<Extractor>
where
    State: Send + Sync,
    Args: Send + Sync + FromRef<State>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequest<State>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v, Args = &'v Args>,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State>>::Rejection>;

    async fn from_request(req: Request, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;

        inner.get_validate_args().validate_with_args(&arguments)?;
        Ok(ValidEx(inner))
    }
}

impl<State, Extractor, Args> FromRequestParts<State> for ValidEx<Extractor>
where
    State: Send + Sync,
    Args: Send + Sync + FromRef<State>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequestParts<State>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v, Args = &'v Args>,
{
    type Rejection = ValidRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate_args().validate_with_args(&arguments)?;
        Ok(ValidEx(inner))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::error::Error;
    use std::fmt::Formatter;
    use std::io;
    use validator::ValidationError;
    const TEST: &str = "test";

    #[test]
    fn valid_deref_deref_mut_into_inner() {
        let mut inner = String::from(TEST);
        let mut v = Valid(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(TEST);
        v.deref_mut().push_str(TEST);
        assert_eq!(&inner, v.deref());
        println!("{}", v);
        assert_eq!(inner, v.into_inner());
    }

    #[test]
    fn valid_ex_deref_deref_mut_into_inner_arguments() {
        let mut inner = String::from(TEST);
        let mut v = ValidEx(inner.clone());
        assert_eq!(&inner, v.deref());
        inner.push_str(TEST);
        v.deref_mut().push_str(TEST);
        assert_eq!(&inner, v.deref());
        assert_eq!(inner, v.into_inner());

        fn validate(v: i32, args: &DataVA) -> Result<(), ValidationError> {
            assert!(v < args.a);
            Ok(())
        }

        #[derive(Debug, Validate)]
        #[validate(context = DataVA)]
        struct Data {
            #[validate(custom(function = "validate", use_context))]
            v: i32,
        }

        impl Display for Data {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }

        struct DataVA {
            a: i32,
        }

        let v = 12;
        let data = Data { v };
        let args = DataVA { a: v + 1 };
        let ve = ValidEx(data);
        ve.validate_with_args(&args).expect("invalid");
        println!("{}", ve);
        assert_eq!(ve.v, v);
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
