//! # Support for `T: TypedPath` from `axum-extra`
//!
//! ## Feature
//!
//! Enable the `extra_typed_path` feature to use `Valid<T: TypedPath>`.
//!
//! ## Usage
//!
//! 1. Implement `TypedPath`, `Deserialize`, `Validate` and `HasValidate` for your data type `T`.
//! 2. In your handler function, use `Valid<T>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::Router;
//!     use axum_extra::routing::{RouterExt, TypedPath};
//!     use axum_valid::{HasValidate, Valid};
//!     use serde::Deserialize;
//!     use validator::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().typed_get(handler)
//!     }
//!
//!     async fn handler(parameter: Valid<Parameter>) {
//!         assert!(parameter.validate().is_ok());
//!     }
//!
//!     #[derive(TypedPath, Deserialize, Validate)]
//!     #[typed_path("/extra_typed_path/{v0}/{v1}")]
//!     struct Parameter {
//!         #[validate(range(min = 5, max = 10))]
//!         v0: i32,
//!         #[validate(length(min = 1, max = 10))]
//!         v1: String,
//!     }
//!
//!     impl HasValidate for Parameter {
//!         type Validate = Self;
//!
//!         fn get_validate(&self) -> &Self::Validate {
//!             self
//!         }
//!     }
//! }
//!
//! #[cfg(feature = "garde")]
//! mod garde_example {
//!     use axum::Router;
//!     use axum_extra::routing::{RouterExt, TypedPath};
//!     use axum_valid::{HasValidate, Garde};
//!     use serde::Deserialize;
//!     use garde::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new().typed_get(handler)
//!     }
//!
//!     async fn handler(parameter: Garde<Parameter>) {
//!         assert!(parameter.validate_with(&()).is_ok());
//!     }
//!
//!     #[derive(TypedPath, Deserialize, Validate)]
//!     #[typed_path("/extra_typed_path/{v0}/{v1}")]
//!     struct Parameter {
//!         #[garde(range(min = 5, max = 10))]
//!         v0: i32,
//!         #[garde(length(min = 1, max = 10))]
//!         v1: String,
//!     }
//!
//!     impl HasValidate for Parameter {
//!         type Validate = Self;
//!
//!         fn get_validate(&self) -> &Self::Validate {
//!             self
//!         }
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

#[cfg(feature = "garde")]
use crate::Garde;
#[cfg(feature = "validify")]
use crate::{Modified, Validated, ValidifiedByRef};
#[cfg(feature = "validator")]
use crate::{Valid, ValidEx};
#[cfg(any(feature = "validator", feature = "garde", feature = "validify"))]
use axum_extra::routing::TypedPath;
#[cfg(any(feature = "validator", feature = "garde", feature = "validify"))]
use std::fmt::Display;

#[cfg(feature = "validator")]
impl<T: TypedPath + Display> TypedPath for Valid<T> {
    const PATH: &'static str = T::PATH;
}

#[cfg(feature = "validator")]
impl<T: TypedPath + Display> TypedPath for ValidEx<T> {
    const PATH: &'static str = T::PATH;
}

#[cfg(feature = "garde")]
impl<T: TypedPath + Display> TypedPath for Garde<T> {
    const PATH: &'static str = T::PATH;
}

#[cfg(feature = "validify")]
impl<T: TypedPath + Display> TypedPath for Validated<T> {
    const PATH: &'static str = T::PATH;
}

#[cfg(feature = "validify")]
impl<T: TypedPath + Display> TypedPath for Modified<T> {
    const PATH: &'static str = T::PATH;
}

#[cfg(feature = "validify")]
impl<T: TypedPath + Display> TypedPath for ValidifiedByRef<T> {
    const PATH: &'static str = T::PATH;
}
