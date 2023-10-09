//! # Validator support

#[cfg(test)]
pub mod test;

use crate::{HasValidate, VALIDATION_ERROR_STATUS};
use axum::async_trait;
use axum::extract::{FromRef, FromRequest, FromRequestParts};
use axum::http::request::Parts;
use axum::http::Request;
use axum::response::{IntoResponse, Response};
use std::error::Error;
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

impl<T: Display, A> Display for ValidEx<T, A> {
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<E, A> ValidEx<E, A> {
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
    pub fn arguments<'a>(&'a self) -> <<A as Arguments<'a>>::T as ValidateArgs<'a>>::Args
    where
        A: Arguments<'a>,
    {
        self.1.get()
    }
}

fn response_builder(ve: ValidationErrors) -> Response {
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
pub trait Arguments<'a> {
    /// The data type to validate using this arguments
    type T: ValidateArgs<'a>;
    /// This method gets the arguments required by `ValidateArgs::validate_args`
    fn get(&'a self) -> <<Self as Arguments<'a>>::T as ValidateArgs<'a>>::Args;
}

/// `ValidRejection` is returned when the `Valid` extractor fails.
///
/// This enumeration captures two types of errors that can occur when using `Valid`: errors related to the validation
/// logic itself (encapsulated in `Valid`), and errors that may arise within the inner extractor (represented by `Inner`).
///
#[derive(Debug)]
pub enum ValidRejection<E> {
    /// `Valid` variant captures errors related to the validation logic. It contains `ValidationErrors`
    /// which is a collection of validation failures for each field.
    Valid(ValidationErrors),
    /// `Inner` variant represents potential errors that might occur within the inner extractor.
    Inner(E),
}

impl<E: Display> Display for ValidRejection<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidRejection::Valid(errors) => write!(f, "{errors}"),
            ValidRejection::Inner(error) => write!(f, "{error}"),
        }
    }
}

impl<E: Error + 'static> Error for ValidRejection<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ValidRejection::Valid(ve) => Some(ve),
            ValidRejection::Inner(e) => Some(e),
        }
    }
}

impl<E> From<ValidationErrors> for ValidRejection<E> {
    fn from(value: ValidationErrors) -> Self {
        Self::Valid(value)
    }
}

impl<E: IntoResponse> IntoResponse for ValidRejection<E> {
    fn into_response(self) -> Response {
        match self {
            ValidRejection::Valid(ve) => response_builder(ve),
            ValidRejection::Inner(e) => e.into_response(),
        }
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

#[async_trait]
impl<State, Body, Extractor> FromRequest<State, Body> for Valid<Extractor>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Extractor: HasValidate + FromRequest<State, Body>,
    Extractor::Validate: Validate,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate().validate()?;
        Ok(Valid(inner))
    }
}

#[async_trait]
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

#[async_trait]
impl<State, Body, Extractor, Args> FromRequest<State, Body> for ValidEx<Extractor, Args>
where
    State: Send + Sync,
    Body: Send + Sync + 'static,
    Args: Send
        + Sync
        + FromRef<State>
        + for<'a> Arguments<'a, T = <Extractor as HasValidateArgs<'a>>::ValidateArgs>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequest<State, Body>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v>,
{
    type Rejection = ValidRejection<<Extractor as FromRequest<State, Body>>::Rejection>;

    async fn from_request(req: Request<Body>, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request(req, state)
            .await
            .map_err(ValidRejection::Inner)?;

        inner.get_validate_args().validate_args(arguments.get())?;
        Ok(ValidEx(inner, arguments))
    }
}

#[async_trait]
impl<State, Extractor, Args> FromRequestParts<State> for ValidEx<Extractor, Args>
where
    State: Send + Sync,
    Args: Send
        + Sync
        + FromRef<State>
        + for<'a> Arguments<'a, T = <Extractor as HasValidateArgs<'a>>::ValidateArgs>,
    Extractor: for<'v> HasValidateArgs<'v> + FromRequestParts<State>,
    for<'v> <Extractor as HasValidateArgs<'v>>::ValidateArgs: ValidateArgs<'v>,
{
    type Rejection = ValidRejection<<Extractor as FromRequestParts<State>>::Rejection>;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection> {
        let arguments: Args = FromRef::from_ref(state);
        let inner = Extractor::from_request_parts(parts, state)
            .await
            .map_err(ValidRejection::Inner)?;
        inner.get_validate_args().validate_args(arguments.get())?;
        Ok(ValidEx(inner, arguments))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
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
        assert_eq!(inner, v.into_inner());
    }

    #[test]
    fn valid_ex_deref_deref_mut_into_inner_arguments() {
        let mut inner = String::from(TEST);
        let mut v = ValidEx(inner.clone(), ());
        assert_eq!(&inner, v.deref());
        inner.push_str(TEST);
        v.deref_mut().push_str(TEST);
        assert_eq!(&inner, v.deref());
        assert_eq!(inner, v.into_inner());

        fn validate(_v: i32, _args: i32) -> Result<(), ValidationError> {
            Ok(())
        }

        #[derive(Validate)]
        struct Data {
            #[validate(custom(function = "validate", arg = "i32"))]
            v: i32,
        }

        struct DataVA {
            a: i32,
        }

        impl<'a> Arguments<'a> for DataVA {
            type T = Data;

            fn get(&'a self) -> <<Self as Arguments<'a>>::T as ValidateArgs<'a>>::Args {
                self.a
            }
        }

        let data = Data { v: 12 };
        let args = DataVA { a: 123 };
        let ve = ValidEx(data, args);
        assert_eq!(ve.v, 12);
        let a = ve.arguments();
        assert_eq!(a, 123);
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
