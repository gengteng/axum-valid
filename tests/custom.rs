//! # Custom extractor validation
//!

#![cfg(feature = "validator")]

use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_valid::{HasValidate, Valid, VALIDATION_ERROR_STATUS};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use validator::Validate;

const MY_DATA_HEADER: &str = "My-Data";

// 1. Implement your own extractor.
//  1.1. Define you own extractor type.
#[derive(Debug, Serialize, Deserialize, Validate)]
struct MyData {
    #[validate(length(min = 1, max = 10))]
    content: String,
}

//  1.2. Define you own `Rejection` type and implement `IntoResponse` for it.
enum MyDataRejection {
    Null,
    InvalidJson(serde_json::error::Error),
}

impl IntoResponse for MyDataRejection {
    fn into_response(self) -> Response {
        match self {
            MyDataRejection::Null => {
                (StatusCode::BAD_REQUEST, "My-Data header is missing").into_response()
            }
            MyDataRejection::InvalidJson(e) => (
                StatusCode::BAD_REQUEST,
                format!("My-Data is not valid json string: {e}"),
            )
                .into_response(),
        }
    }
}

//  1.3. Implement your extractor (`FromRequestParts` or `FromRequest`)
#[axum::async_trait]
impl<S> FromRequestParts<S> for MyData
where
    S: Send + Sync,
{
    type Rejection = MyDataRejection;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let Some(value) = parts.headers.get(MY_DATA_HEADER) else {
            return Err(MyDataRejection::Null);
        };

        serde_json::from_slice(value.as_bytes()).map_err(MyDataRejection::InvalidJson)
    }
}

// 2. Use axum-valid to validate the extractor
//  2.1. Implement `HasValidate` for your extractor
impl HasValidate for MyData {
    type Validate = Self;
    fn get_validate(&self) -> &Self::Validate {
        self
    }
}

#[tokio::test]
async fn main() -> anyhow::Result<()> {
    let router = Router::new().route("/", get(handler));

    let listener = TcpListener::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16))).await?;
    let server_addr = listener.local_addr()?;
    let server = axum::serve(listener, router.into_make_service());
    println!("Axum server address: {}.", server_addr);

    // let (server_guard, close) = tokio::sync::oneshot::channel::<()>();
    tokio::spawn(async move {
        let _ = server.await;
    });

    let client = reqwest::Client::default();
    let url = format!("http://{}/", server_addr);

    let valid_my_data = MyData {
        content: String::from("hello"),
    };
    let valid_my_data_response = client
        .get(&url)
        .header(MY_DATA_HEADER, serde_json::to_string(&valid_my_data)?)
        .send()
        .await?;
    assert_eq!(
        valid_my_data_response.status().as_u16(),
        StatusCode::OK.as_u16()
    );

    let invalid_json = String::from("{{}");
    let valid_my_data_response = client
        .get(&url)
        .header(MY_DATA_HEADER, invalid_json)
        .send()
        .await?;
    assert_eq!(
        valid_my_data_response.status().as_u16(),
        StatusCode::BAD_REQUEST.as_u16()
    );

    let invalid_my_data = MyData {
        content: String::new(),
    };
    let invalid_my_data_response = client
        .get(&url)
        .header(MY_DATA_HEADER, serde_json::to_string(&invalid_my_data)?)
        .send()
        .await?;
    assert_eq!(
        invalid_my_data_response.status().as_u16(),
        VALIDATION_ERROR_STATUS.as_u16()
    );
    // #[cfg(feature = "into_json")]
    // test::check_json(invalid_my_data_response).await;
    println!("Valid<MyData> works.");

    // drop(server_guard);
    // server_handle.await??;
    Ok(())
}

async fn handler(Valid(my_data): Valid<MyData>) -> StatusCode {
    match my_data.validate() {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
