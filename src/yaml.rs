//! # Support for `Yaml<T>` from `axum-yaml`
//!
//! ## Feature
//!
//! Enable the `yaml` feature to use `Valid<Yaml<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<Yaml<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! #![cfg(feature = "validator")]
//!
//! use axum::routing::post;
//! use axum::Router;
//! use axum_valid::Valid;
//! use axum_yaml::Yaml;
//! use serde::Deserialize;
//! use validator::Validate;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let router = Router::new().route("/yaml", post(handler));
//!     axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
//!         .serve(router.into_make_service())
//!         .await?;
//!     Ok(())
//! }
//!
//! async fn handler(parameter: Valid<Yaml<Parameter>>) {
//!     assert!(parameter.validate().is_ok());
//! }
//!
//! #[derive(Deserialize, Validate)]
//! struct Parameter {
//!     #[validate(range(min = 5, max = 10))]
//!     v0: i32,
//!     #[validate(length(min = 1, max = 10))]
//!     v1: String,
//! }
//! ```

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_yaml::Yaml;
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for Yaml<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for Yaml<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_yaml::Yaml;
    use reqwest::RequestBuilder;
    use serde::Serialize;

    impl<T: ValidTestParameter + Serialize> ValidTest for Yaml<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNSUPPORTED_MEDIA_TYPE;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/yaml")
                .body(
                    serde_yaml::to_string(&T::valid())
                        .expect("Failed to serialize parameters to yaml"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            // `Content-Type` not set, `Yaml` should return `415 Unsupported Media Type`
            builder
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/yaml")
                .body(
                    serde_yaml::to_string(&T::invalid())
                        .expect("Failed to serialize parameters to yaml"),
                )
        }
    }
}
