//! # Support for `Form<T>`
//!
//! ## Feature
//!
//! Enable the `form` feature (enabled by default) to use `Valid<Form<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Form<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::routing::post;
//!     use axum::Form;
//!     use axum::Router;
//!     use axum_valid::Valid;
//!     use serde::Deserialize;
//!     use validator::Validate;
//!     #[tokio::main]
//!     pub async fn launch() -> anyhow::Result<()> {
//!         let router = Router::new().route("/json", post(handler));
//!         axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!             .serve(router.into_make_service())
//!             .await?;
//!         Ok(())
//!     }
//!     async fn handler(Valid(Form(parameter)): Valid<Form<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!     #[derive(Validate, Deserialize)]
//!     pub struct Parameter {
//!         #[validate(range(min = 5, max = 10))]
//!         pub v0: i32,
//!         #[validate(length(min = 1, max = 10))]
//!         pub v1: String,
//!     }
//! }
//!
//! #[cfg(feature = "garde")]
//! mod garde_example {
//!     use axum::routing::post;
//!     use axum::Form;
//!     use axum::Router;
//!     use axum_valid::Garde;
//!     use serde::Deserialize;
//!     use garde::Validate;
//!     #[tokio::main]
//!     pub async fn launch() -> anyhow::Result<()> {
//!         let router = Router::new().route("/json", post(handler));
//!         axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!             .serve(router.into_make_service())
//!             .await?;
//!         Ok(())
//!     }
//!     async fn handler(Garde(Form(parameter)): Garde<Form<Parameter>>) {
//!         assert!(parameter.validate(&()).is_ok());
//!         // Support automatic dereferencing
//!         println!("v0 = {}, v1 = {}", parameter.v0, parameter.v1);
//!     }
//!     #[derive(Validate, Deserialize)]
//!     pub struct Parameter {
//!         #[garde(range(min = 5, max = 10))]
//!         pub v0: i32,
//!         #[garde(length(min = 1, max = 10))]
//!         pub v1: String,
//!     }
//! }
//!
//! # fn main() -> anyhow::Result<()> {
//! #     #[cfg(feature = "validator")]
//! #     validator_example::launch()?;
//! #     #[cfg(feature = "garde")]
//! #     garde_example::launch()?;
//! #     Ok(())
//! # }
//! ```

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum::Form;
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for Form<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Form<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum::Form;
    use reqwest::RequestBuilder;
    use serde::Serialize;

    impl<T: ValidTestParameter + Serialize> ValidTest for Form<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::valid())
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::error())
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.form(T::invalid())
        }
    }
}
