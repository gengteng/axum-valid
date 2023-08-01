/// Check if the response is a json response
#[cfg(feature = "into_json")]
pub async fn check_json(response: reqwest::Response) {
    assert_eq!(
        response.headers()[axum::http::header::CONTENT_TYPE],
        axum::http::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref())
    );
    assert!(response.json::<serde_json::Value>().await.is_ok());
}
