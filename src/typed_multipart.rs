//! # Support for `TypedMultipart<T>` and `BaseMultipart<T, R>` from `axum_typed_multipart`
//!
//! ## Feature
//!
//! Enable the `typed_multipart` feature to use `Valid<TypedMultipart<T>>` and `Valid<BaseMultipart<T, R>>`.
//!
//! ## Usage
//!
//! 1. Implement `TryFromMultipart` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<TypedMultipart<T>>` or `Valid<BaseMultipart<T, E>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::routing::post;
//!     use axum::Router;
//!     use axum_typed_multipart::{BaseMultipart, TryFromMultipart, TypedMultipart, TypedMultipartError};
//!     use axum_valid::Valid;
//!     use validator::Validate;
//!
//!     pub fn router() -> Router {
//!        Router::new()
//!             .route("/typed_multipart", post(handler))
//!             .route("/base_multipart", post(base_handler))
//!     }
//!
//!     async fn handler(Valid(TypedMultipart(parameter)): Valid<TypedMultipart<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!
//!     async fn base_handler(
//!         Valid(BaseMultipart {
//!             data: parameter, ..
//!         }): Valid<BaseMultipart<Parameter, TypedMultipartError>>,
//!     ) {
//!         assert!(parameter.validate().is_ok());
//!     }
//!
//!     #[derive(TryFromMultipart, Validate)]
//!     struct Parameter {
//!         #[validate(range(min = 5, max = 10))]
//!         v0: i32,
//!         #[validate(length(min = 1, max = 10))]
//!         v1: String,
//!     }
//! }
//!
//! #[cfg(feature = "garde")]
//! mod garde_example {
//!     use axum::routing::post;
//!     use axum::Router;
//!     use axum_typed_multipart::{BaseMultipart, TryFromMultipart, TypedMultipart, TypedMultipartError};
//!     use axum_valid::Garde;
//!     use serde::Deserialize;
//!     use garde::Validate;
//!
//!     pub fn router() -> Router {
//!        Router::new()
//!             .route("/typed_multipart", post(handler))
//!             .route("/base_multipart", post(base_handler))
//!     }
//!
//!     async fn handler(Garde(TypedMultipart(parameter)): Garde<TypedMultipart<Parameter>>) {
//!         assert!(parameter.validate_with(&()).is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!
//!     async fn base_handler(
//!         Garde(BaseMultipart {
//!             data: parameter, ..
//!         }): Garde<BaseMultipart<Parameter, TypedMultipartError>>,
//!     ) {
//!         assert!(parameter.validate_with(&()).is_ok());
//!     }
//!
//!     #[derive(TryFromMultipart, Validate)]
//!     pub struct Parameter {
//!         #[garde(range(min = 5, max = 10))]
//!         pub v0: i32,
//!         #[garde(length(min = 1, max = 10))]
//!         pub v1: String,
//!     }
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! #     use std::net::SocketAddr;
//! #     use axum::Router;
//! #     use tokio::net::TcpListener;
//! #     let router = Router::new();
//! #     #[cfg(feature = "validator")]
//! #     let router = router.nest("/validator", validator_example::router());
//! #     #[cfg(feature = "garde")]
//! #     let router = router.nest("/garde", garde_example::router());
//! #     let listener = TcpListener::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16))).await?;
//! #     axum::serve(listener, router.into_make_service())
//! #         .await?;
//! #     Ok(())
//! # }
//! ```

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_typed_multipart::{BaseMultipart, TypedMultipart};
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T, R> HasValidate for BaseMultipart<T, R> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.data
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>, R> HasValidateArgs<'v> for BaseMultipart<T, R> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.data
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Modify, R> crate::HasModify for BaseMultipart<T, R> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.data
    }
}

impl<T> HasValidate for TypedMultipart<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for TypedMultipart<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Modify> crate::HasModify for TypedMultipart<T> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_typed_multipart::{BaseMultipart, TypedMultipart};
    use reqwest::multipart::Form;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter, R> ValidTest for BaseMultipart<T, R>
    where
        Form: From<&'static T>,
    {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::valid()))
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::new())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::invalid()))
        }
    }

    impl<T: ValidTestParameter> ValidTest for TypedMultipart<T>
    where
        Form: From<&'static T>,
    {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::BAD_REQUEST;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::valid()))
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::new())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.multipart(Form::from(T::invalid()))
        }
    }
}
