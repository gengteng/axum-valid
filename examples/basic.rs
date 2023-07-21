use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use axum_valid::Valid;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use validator::Validate;

mod route {
    pub const PATH: &'static str = "/path/:v0/:v1";
    pub const QUERY: &'static str = "/query";
    pub const FORM: &'static str = "/form";
    pub const JSON: &'static str = "/json";
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .route(route::PATH, get(extract_path))
        .route(route::QUERY, get(extract_query))
        .route(route::FORM, post(extract_form))
        .route(route::JSON, post(extract_json));

    let server = axum::Server::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16)))
        .serve(router.into_make_service());
    let server_addr = server.local_addr();
    println!("Axum server address: {}.", server_addr);

    let (server_guard, close) = tokio::sync::oneshot::channel::<()>();
    let server_handle = tokio::spawn(server.with_graceful_shutdown(async move {
        let _ = close.await;
    }));

    let server_url = format!("http://{}", server_addr);
    let client = reqwest::Client::default();

    let valid_parameters = Parameters {
        v0: 5,
        v1: "0123456789".to_string(),
    };

    let invalid_parameters = Parameters {
        v0: 6,
        v1: "01234567890".to_string(),
    };

    // Valid<Path<...>>
    let valid_path_response = client
        .get(format!(
            "{}/path/{}/{}",
            server_url, valid_parameters.v0, valid_parameters.v1
        ))
        .send()
        .await?;
    assert_eq!(valid_path_response.status(), StatusCode::OK);

    let invalid_path_response = client
        .get(format!(
            "{}/path/{}/{}",
            server_url, invalid_parameters.v0, invalid_parameters.v1
        ))
        .send()
        .await?;
    assert_eq!(invalid_path_response.status(), StatusCode::BAD_REQUEST);
    println!("Valid<Path<...>> works.");

    // Valid<Query<...>>
    let query_url = format!("{}{}", server_url, route::QUERY);
    let valid_query_response = client
        .get(&query_url)
        .query(&valid_parameters)
        .send()
        .await?;
    assert_eq!(valid_query_response.status(), StatusCode::OK);

    let invalid_query_response = client
        .get(&query_url)
        .query(&invalid_parameters)
        .send()
        .await?;
    assert_eq!(invalid_query_response.status(), StatusCode::BAD_REQUEST);
    println!("Valid<Query<...>> works.");

    // Valid<Form<...>>
    let form_url = format!("{}{}", server_url, route::FORM);
    let valid_form_response = client
        .post(&form_url)
        .form(&valid_parameters)
        .send()
        .await?;
    assert_eq!(valid_form_response.status(), StatusCode::OK);

    let invalid_form_response = client
        .post(&form_url)
        .form(&invalid_parameters)
        .send()
        .await?;
    assert_eq!(invalid_form_response.status(), StatusCode::BAD_REQUEST);
    println!("Valid<Form<...>> works.");

    // Valid<Json<...>>
    let json_url = format!("{}{}", server_url, route::JSON);
    let valid_json_response = client
        .post(&json_url)
        .json(&valid_parameters)
        .send()
        .await?;
    assert_eq!(valid_json_response.status(), StatusCode::OK);

    let invalid_json_response = client
        .post(&json_url)
        .json(&invalid_parameters)
        .send()
        .await?;
    assert_eq!(invalid_json_response.status(), StatusCode::BAD_REQUEST);
    println!("Valid<Json<...>> works.");

    drop(server_guard);
    server_handle.await??;
    Ok(())
}

// Implement `Deserialize` and `Validate` for `Parameters`,
// then `Valid` will work as you expect.
#[derive(Debug, Deserialize, Serialize, Validate)]
struct Parameters {
    #[validate(range(min = 5, max = 10))]
    v0: i32,
    #[validate(length(min = 1, max = 10))]
    v1: String,
}

async fn extract_path(Valid(Path(parameters)): Valid<Path<Parameters>>) -> StatusCode {
    validate_again(parameters)
}

async fn extract_query(Valid(Query(parameters)): Valid<Query<Parameters>>) -> StatusCode {
    validate_again(parameters)
}

async fn extract_form(Valid(Form(parameters)): Valid<Form<Parameters>>) -> StatusCode {
    validate_again(parameters)
}

async fn extract_json(Valid(Json(parameters)): Valid<Json<Parameters>>) -> StatusCode {
    validate_again(parameters)
}

fn validate_again(parameters: Parameters) -> StatusCode {
    // The `Valid` extractor has validated the `parameters` once,
    // it should have returned `400 BAD REQUEST` if the `parameters` were invalid,
    // Let's validate them again to check if the `Valid` extractor works well.
    // If it works properly, this function will never return `500 INTERNAL SERVER ERROR`
    match parameters.validate() {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
