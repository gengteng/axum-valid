//! # Support for `MsgPack<T>` and `MsgPackRaw<T>` from `axum-serde`
//!
//! ## Feature
//!
//! Enable the `msgpack` feature to use `Valid<MsgPack<T>>` and `Valid<MsgPackRaw<T>>`.
//!
//! ## Usage
//!
//! 1. Implement `Deserialize` and `Validate` for your data type `T`.
//! 2. In your handler function, use `Valid<MsgPack<T>>` or `Valid<MsgPackRaw<T>>` as some parameter's type.
//!
//! ## Example
//!
//! ```no_run
//! #[cfg(feature = "validator")]
//! mod validator_example {
//!     use axum::routing::post;
//!     use axum::Json;
//!     use axum::Router;
//!     use axum_serde::{MsgPack, MsgPackRaw};
//!     use axum_valid::Valid;
//!     use serde::Deserialize;
//!     use validator::Validate;
//!
//!     pub fn router() -> Router {
//!         Router::new()
//!             .route("/msgpack", post(handler))
//!             .route("/msgpackraw", post(raw_handler))
//!     }
//!     async fn handler(Valid(MsgPack(parameter)): Valid<MsgPack<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
//!     }
//!
//!     async fn raw_handler(Valid(MsgPackRaw(parameter)): Valid<MsgPackRaw<Parameter>>) {
//!         assert!(parameter.validate().is_ok());
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
//!     use axum::Router;
//!     use axum_serde::{MsgPack, MsgPackRaw};
//!     use axum_valid::Garde;
//!     use serde::Deserialize;
//!     use garde::Validate;
//!     
//!     pub fn router() -> Router {
//!         Router::new()
//!             .route("/msgpack", post(handler))
//!             .route("/msgpackraw", post(raw_handler))
//!     }
//!
//!     async fn handler(Garde(MsgPack(parameter)): Garde<MsgPack<Parameter>>) {
//!         assert!(parameter.validate_with(&()).is_ok());
//!     }
//!
//!     async fn raw_handler(Garde(MsgPackRaw(parameter)): Garde<MsgPackRaw<Parameter>>) {
//!         assert!(parameter.validate_with(&()).is_ok());
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
//!

use crate::HasValidate;
#[cfg(feature = "validator")]
use crate::HasValidateArgs;
use axum_serde::{MsgPack, MsgPackRaw};
#[cfg(feature = "validator")]
use validator::ValidateArgs;

impl<T> HasValidate for MsgPack<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for MsgPack<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Modify> crate::HasModify for MsgPack<T> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.0
    }
}

#[cfg(feature = "validify")]
impl<T> crate::PayloadExtractor for MsgPack<T> {
    type Payload = T;

    fn get_payload(self) -> Self::Payload {
        self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Validify + validify::ValidifyPayload> crate::HasValidify for MsgPack<T> {
    type Validify = T;
    type PayloadExtractor = MsgPack<T::Payload>;
    fn from_validify(v: Self::Validify) -> Self {
        MsgPack(v)
    }
}

impl<T> HasValidate for MsgPackRaw<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(feature = "validator")]
impl<'v, T: ValidateArgs<'v>> HasValidateArgs<'v> for MsgPackRaw<T> {
    type ValidateArgs = T;
    fn get_validate_args(&self) -> &Self::ValidateArgs {
        &self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Modify> crate::HasModify for MsgPackRaw<T> {
    type Modify = T;

    fn get_modify(&mut self) -> &mut Self::Modify {
        &mut self.0
    }
}

#[cfg(feature = "validify")]
impl<T> crate::PayloadExtractor for MsgPackRaw<T> {
    type Payload = T;

    fn get_payload(self) -> Self::Payload {
        self.0
    }
}

#[cfg(feature = "validify")]
impl<T: validify::Validify + validify::ValidifyPayload> crate::HasValidify for MsgPackRaw<T> {
    type Validify = T;
    type PayloadExtractor = MsgPackRaw<T::Payload>;
    fn from_validify(v: Self::Validify) -> Self {
        MsgPackRaw(v)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_serde::{MsgPack, MsgPackRaw};
    use reqwest::RequestBuilder;
    use serde::Serialize;

    impl<T: ValidTestParameter + Serialize> ValidTest for MsgPack<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            #[derive(Serialize, Default)]
            struct ErrorData {
                error_field0: i32,
                error_field1: Option<String>,
            }
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(&ErrorData::default())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec_named(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }

    impl<T: ValidTestParameter + Serialize> ValidTest for MsgPackRaw<T> {
        const ERROR_STATUS_CODE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::valid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            #[derive(Serialize, Default)]
            struct ErrorData {
                error_field0: i32,
                error_field1: Option<String>,
            }
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(&ErrorData::default())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder
                .header(reqwest::header::CONTENT_TYPE, "application/msgpack")
                .body(
                    rmp_serde::to_vec(T::invalid())
                        .expect("Failed to serialize parameters to msgpack"),
                )
        }
    }
}
