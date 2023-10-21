#![cfg(feature = "validify")]

use crate::tests::{ValidTest, ValidTestParameter};
use crate::{
    HasValidate, Modified, Validated, Validified, ValidifiedByRef, VALIDATION_ERROR_STATUS,
};
use axum::extract::{Path, Query};
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use hyper::Method;
use once_cell::sync::Lazy;
use reqwest::{StatusCode, Url};
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::net::SocketAddr;
use std::ops::Deref;
use validify::{Modify, Validate, Validify};

#[derive(Clone, Deserialize, Serialize, Validify, Eq, PartialEq)]
#[cfg_attr(feature = "extra_protobuf", derive(prost::Message))]
#[cfg_attr(
    feature = "typed_multipart",
    derive(axum_typed_multipart::TryFromMultipart)
)]
pub struct ParametersValidify {
    #[validate(range(min = 5.0, max = 10.0))]
    #[cfg_attr(feature = "extra_protobuf", prost(int32, tag = "1"))]
    v0: i32,
    #[modify(lowercase)]
    #[validate(length(min = 1, max = 10))]
    #[cfg_attr(feature = "extra_protobuf", prost(string, tag = "2"))]
    v1: String,
}

trait IsModified: Modify + Clone + PartialEq + Eq {
    fn modified(&self) -> bool {
        let mut cloned = self.clone();
        cloned.modify();
        cloned == *self
    }
}

impl<T> IsModified for T where T: Modify + Clone + PartialEq + Eq {}

static VALID_PARAMETERS: Lazy<ParametersValidify> = Lazy::new(|| ParametersValidify {
    v0: 5,
    v1: String::from("ABCDEFG"),
});

static INVALID_PARAMETERS: Lazy<ParametersValidify> = Lazy::new(|| ParametersValidify {
    v0: 6,
    v1: String::from("ABCDEFGHIJKLMN"),
});

impl ValidTestParameter for ParametersValidify {
    fn valid() -> &'static Self {
        VALID_PARAMETERS.deref()
    }

    fn error() -> &'static [(&'static str, &'static str)] {
        &[("not_v0_or_v1", "value")]
    }

    fn invalid() -> &'static Self {
        INVALID_PARAMETERS.deref()
    }
}

impl HasValidate for ParametersValidify {
    type Validate = ParametersValidify;

    fn get_validate(&self) -> &Self::Validate {
        self
    }
}

#[tokio::test]
async fn test_main() -> anyhow::Result<()> {
    let router = Router::new()
        .route(route::PATH, get(extract_path))
        .route(route::PATH_MODIFIED, get(extract_path_modified))
        .route(route::PATH_VALIDIFIED, get(extract_path_validified))
        .route(
            route::PATH_VALIDIFIED_BY_REF,
            get(extract_path_validified_by_ref),
        )
        .route(route::QUERY, get(extract_query))
        .route(route::QUERY_MODIFIED, get(extract_query_modified))
        .route(route::QUERY_VALIDIFIED, get(extract_query_validified))
        .route(
            route::QUERY_VALIDIFIED_BY_REF,
            get(extract_query_validified_by_ref),
        )
        .route(route::FORM, post(extract_form))
        .route(route::FORM_MODIFIED, post(extract_form_modified))
        .route(route::FORM_VALIDIFIED, post(extract_form_validified))
        .route(
            route::FORM_VALIDIFIED_BY_REF,
            post(extract_form_validified_by_ref),
        )
        .route(route::JSON, post(extract_json))
        .route(route::JSON_MODIFIED, post(extract_json_modified))
        .route(route::JSON_VALIDIFIED, post(extract_json_validified))
        .route(
            route::JSON_VALIDIFIED_BY_REF,
            post(extract_json_validified_by_ref),
        );

    #[cfg(feature = "typed_header")]
    let router = router
        .route(
            typed_header::route::TYPED_HEADER,
            post(typed_header::extract_typed_header),
        )
        .route(
            typed_header::route::TYPED_HEADER_MODIFIED,
            post(typed_header::extract_typed_header_modified),
        )
        .route(
            typed_header::route::TYPED_HEADER_VALIDIFIED_BY_REF,
            post(typed_header::extract_typed_header_validified_by_ref),
        );

    #[cfg(feature = "typed_multipart")]
    let router = router
        .route(
            typed_multipart::route::TYPED_MULTIPART,
            post(typed_multipart::extract_typed_multipart),
        )
        .route(
            typed_multipart::route::TYPED_MULTIPART_MODIFIED,
            post(typed_multipart::extract_typed_multipart_modified),
        )
        .route(
            typed_multipart::route::TYPED_MULTIPART_VALIDIFIED_BY_REF,
            post(typed_multipart::extract_typed_multipart_validified_by_ref),
        )
        .route(
            typed_multipart::route::BASE_MULTIPART,
            post(typed_multipart::extract_base_multipart),
        )
        .route(
            typed_multipart::route::BASE_MULTIPART_MODIFIED,
            post(typed_multipart::extract_base_multipart_modified),
        )
        .route(
            typed_multipart::route::BASE_MULTIPART_VALIDIFIED_BY_REF,
            post(typed_multipart::extract_base_multipart_validified_by_ref),
        );

    #[cfg(feature = "extra")]
    let router = router
        .route(extra::route::CACHED, post(extra::extract_cached))
        .route(
            extra::route::WITH_REJECTION,
            post(extra::extract_with_rejection),
        )
        .route(
            extra::route::WITH_REJECTION_VALIDIFY,
            post(extra::extract_with_rejection_valid),
        );

    #[cfg(feature = "extra_typed_path")]
    let router = router
        .route(
            extra_typed_path::route::EXTRA_TYPED_PATH,
            get(extra_typed_path::extract_extra_typed_path),
        )
        .route(
            extra_typed_path::route::EXTRA_TYPED_PATH_MODIFIED,
            get(extra_typed_path::extract_extra_typed_path_modified),
        )
        .route(
            extra_typed_path::route::EXTRA_TYPED_PATH_VALIDIFIED_BY_REF,
            get(extra_typed_path::extract_extra_typed_path_validified_by_ref),
        );

    #[cfg(feature = "extra_query")]
    let router = router.route(
        extra_query::route::EXTRA_QUERY,
        post(extra_query::extract_extra_query),
    );

    #[cfg(feature = "extra_form")]
    let router = router.route(
        extra_form::route::EXTRA_FORM,
        post(extra_form::extract_extra_form),
    );

    #[cfg(feature = "extra_protobuf")]
    let router = router
        .route(
            extra_protobuf::route::EXTRA_PROTOBUF,
            post(extra_protobuf::extract_extra_protobuf),
        )
        .route(
            extra_protobuf::route::EXTRA_PROTOBUF_MODIFIED,
            post(extra_protobuf::extract_extra_protobuf_modified),
        )
        .route(
            extra_protobuf::route::EXTRA_PROTOBUF_VALIDIFIED_BY_REF,
            post(extra_protobuf::extract_extra_protobuf_validified_by_ref),
        );

    #[cfg(feature = "yaml")]
    let router = router.route(yaml::route::YAML, post(yaml::extract_yaml));

    #[cfg(feature = "msgpack")]
    let router = router
        .route(msgpack::route::MSGPACK, post(msgpack::extract_msgpack))
        .route(
            msgpack::route::MSGPACK_RAW,
            post(msgpack::extract_msgpack_raw),
        );

    let server = axum::Server::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16)))
        .serve(router.into_make_service());
    let server_addr = server.local_addr();
    println!("Axum server address: {}.", server_addr);

    let (server_guard, close) = tokio::sync::oneshot::channel::<()>();
    let server_handle = tokio::spawn(server.with_graceful_shutdown(async move {
        let _ = close.await;
    }));

    let server_url = format!("http://{}", server_addr);
    let test_executor = TestExecutor::from(Url::parse(&format!("http://{}", server_addr))?);

    async fn test_extra_path(
        test_executor: &TestExecutor,
        route: &str,
        server_url: &str,
    ) -> anyhow::Result<()> {
        do_test_extra_path(
            test_executor,
            route,
            server_url,
            StatusCode::OK,
            StatusCode::BAD_REQUEST,
            VALIDATION_ERROR_STATUS,
            true,
        )
        .await
    }

    async fn test_extra_path_modified(
        test_executor: &TestExecutor,
        route: &str,
        server_url: &str,
    ) -> anyhow::Result<()> {
        do_test_extra_path(
            test_executor,
            route,
            server_url,
            StatusCode::OK,
            StatusCode::BAD_REQUEST,
            StatusCode::OK,
            false,
        )
        .await
    }

    async fn test_extra_path_validified(
        test_executor: &TestExecutor,
        route: &str,
        server_url: &str,
    ) -> anyhow::Result<()> {
        do_test_extra_path(
            test_executor,
            route,
            server_url,
            StatusCode::OK,
            StatusCode::BAD_REQUEST,
            VALIDATION_ERROR_STATUS,
            true,
        )
        .await
    }

    async fn do_test_extra_path(
        test_executor: &TestExecutor,
        route: &str,
        server_url: &str,
        expected_valid_status: StatusCode,
        expected_error_status: StatusCode,
        expected_invalid_status: StatusCode,
        should_check_json: bool,
    ) -> anyhow::Result<()> {
        let path_type_name = type_name::<Path<ParametersValidify>>();
        let valid_path_response = test_executor
            .client()
            .get(format!(
                "{}/{route}/{}/{}",
                server_url, VALID_PARAMETERS.v0, VALID_PARAMETERS.v1
            ))
            .send()
            .await?;
        assert_eq!(
            valid_path_response.status(),
            expected_valid_status,
            "Valid '{}' test failed.",
            path_type_name
        );

        let error_path_response = test_executor
            .client()
            .get(format!("{}/{route}/not_i32/path", server_url))
            .send()
            .await?;
        assert_eq!(
            error_path_response.status(),
            expected_error_status,
            "Error '{}' test failed: {}",
            path_type_name,
            error_path_response.text().await?
        );

        let invalid_path_response = test_executor
            .client()
            .get(format!(
                "{}/{route}/{}/{}",
                server_url, INVALID_PARAMETERS.v0, INVALID_PARAMETERS.v1
            ))
            .send()
            .await?;
        assert_eq!(
            invalid_path_response.status(),
            expected_invalid_status,
            "Invalid '{}' test failed.",
            path_type_name
        );
        #[cfg(feature = "into_json")]
        if should_check_json {
            check_json(path_type_name, invalid_path_response).await;
        }
        println!("All {} tests passed.", path_type_name);
        Ok(())
    }

    test_extra_path(&test_executor, "path", &server_url).await?;
    test_extra_path_modified(&test_executor, "path_modified", &server_url).await?;
    test_extra_path_validified(&test_executor, "path_validified", &server_url).await?;
    test_extra_path(&test_executor, "path_validified_by_ref", &server_url).await?;

    // Validated
    test_executor
        .execute::<Query<ParametersValidify>>(Method::GET, route::QUERY)
        .await?;
    // Modified
    test_executor
        .execute_modified::<Query<ParametersValidify>>(Method::GET, route::QUERY_MODIFIED)
        .await?;
    // ValidifiedByRef
    test_executor
        .execute::<Query<ParametersValidify>>(Method::GET, route::QUERY_VALIDIFIED_BY_REF)
        .await?;
    // Validified
    test_executor
        .execute_validified::<Query<ParametersValidify>>(Method::GET, route::QUERY_VALIDIFIED)
        .await?;

    // Validated
    test_executor
        .execute::<Form<ParametersValidify>>(Method::POST, route::FORM)
        .await?;
    // Modified
    test_executor
        .execute_modified::<Form<ParametersValidify>>(Method::POST, route::FORM_MODIFIED)
        .await?;
    // ValidifiedByRef
    test_executor
        .execute::<Form<ParametersValidify>>(Method::POST, route::FORM_VALIDIFIED_BY_REF)
        .await?;
    // Validified
    test_executor
        .execute_validified::<Form<ParametersValidify>>(Method::POST, route::FORM_VALIDIFIED)
        .await?;

    // Validated
    test_executor
        .execute::<Json<ParametersValidify>>(Method::POST, route::JSON)
        .await?;
    // Modified
    test_executor
        .execute_modified::<Json<ParametersValidify>>(Method::POST, route::JSON_MODIFIED)
        .await?;
    // ValidifiedByRef
    test_executor
        .execute::<Json<ParametersValidify>>(Method::POST, route::JSON_VALIDIFIED_BY_REF)
        .await?;
    // Validified
    test_executor
        .execute_validified::<Json<ParametersValidify>>(Method::POST, route::JSON_VALIDIFIED)
        .await?;

    #[cfg(feature = "typed_header")]
    {
        use axum::TypedHeader;
        // Validated
        test_executor
            .execute::<TypedHeader<ParametersValidify>>(
                Method::POST,
                typed_header::route::TYPED_HEADER,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<TypedHeader<ParametersValidify>>(
                Method::POST,
                typed_header::route::TYPED_HEADER_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<TypedHeader<ParametersValidify>>(
                Method::POST,
                typed_header::route::TYPED_HEADER_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "typed_multipart")]
    {
        use axum_typed_multipart::{BaseMultipart, TypedMultipart, TypedMultipartError};

        // Validated
        test_executor
            .execute::<BaseMultipart<ParametersValidify, TypedMultipartError>>(
                Method::POST,
                typed_multipart::route::BASE_MULTIPART,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<BaseMultipart<ParametersValidify, TypedMultipartError>>(
                Method::POST,
                typed_multipart::route::BASE_MULTIPART_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<BaseMultipart<ParametersValidify, TypedMultipartError>>(
                Method::POST,
                typed_multipart::route::BASE_MULTIPART_VALIDIFIED_BY_REF,
            )
            .await?;

        // Validated
        test_executor
            .execute::<TypedMultipart<ParametersValidify>>(
                Method::POST,
                typed_multipart::route::TYPED_MULTIPART,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<TypedMultipart<ParametersValidify>>(
                Method::POST,
                typed_multipart::route::TYPED_MULTIPART_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<TypedMultipart<ParametersValidify>>(
                Method::POST,
                typed_multipart::route::TYPED_MULTIPART_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "extra")]
    {
        use axum_extra::extract::{Cached, WithRejection};
        use extra::{
            ParametersRejection, ValidifyWithRejectionRejection, WithRejectionValidifyRejection,
        };
        test_executor
            .execute::<Cached<ParametersValidify>>(Method::POST, extra::route::CACHED)
            .await?;
        test_executor
            .execute::<WithRejection<ParametersValidify, ValidifyWithRejectionRejection>>(
                Method::POST,
                extra::route::WITH_REJECTION,
            )
            .await?;
        test_executor
            .execute::<WithRejection<
                Validated<ParametersValidify>,
                WithRejectionValidifyRejection<ParametersRejection>,
            >>(Method::POST, extra::route::WITH_REJECTION_VALIDIFY)
            .await?;
    }

    #[cfg(feature = "extra_typed_path")]
    {
        async fn test_extra_typed_path(
            test_executor: &TestExecutor,
            route: &str,
            server_url: &str,
        ) -> anyhow::Result<()> {
            do_test_extra_typed_path(
                test_executor,
                route,
                server_url,
                StatusCode::OK,
                StatusCode::BAD_REQUEST,
                VALIDATION_ERROR_STATUS,
                true,
            )
            .await
        }

        async fn test_extra_typed_path_modified(
            test_executor: &TestExecutor,
            route: &str,
            server_url: &str,
        ) -> anyhow::Result<()> {
            do_test_extra_typed_path(
                test_executor,
                route,
                server_url,
                StatusCode::OK,
                StatusCode::BAD_REQUEST,
                StatusCode::OK,
                false,
            )
            .await
        }

        async fn do_test_extra_typed_path(
            test_executor: &TestExecutor,
            route: &str,
            server_url: &str,
            expected_valid_status: StatusCode,
            expected_error_status: StatusCode,
            expected_invalid_status: StatusCode,
            should_check_json: bool,
        ) -> anyhow::Result<()> {
            let extra_typed_path_type_name = "T: TypedPath";
            let valid_extra_typed_path_response = test_executor
                .client()
                .get(format!(
                    "{}/{route}/{}/{}",
                    server_url, VALID_PARAMETERS.v0, VALID_PARAMETERS.v1
                ))
                .send()
                .await?;
            assert_eq!(
                valid_extra_typed_path_response.status(),
                expected_valid_status,
                "Validified '{}' test failed.",
                extra_typed_path_type_name
            );

            let error_extra_typed_path_response = test_executor
                .client()
                .get(format!("{}/{route}/not_i32/path", server_url))
                .send()
                .await?;
            assert_eq!(
                error_extra_typed_path_response.status(),
                expected_error_status,
                "Error '{}' test failed.",
                extra_typed_path_type_name
            );

            let invalid_extra_typed_path_response = test_executor
                .client()
                .get(format!(
                    "{}/{route}/{}/{}",
                    server_url, INVALID_PARAMETERS.v0, INVALID_PARAMETERS.v1
                ))
                .send()
                .await?;
            assert_eq!(
                invalid_extra_typed_path_response.status(),
                expected_invalid_status,
                "Invalid '{}' test failed.",
                extra_typed_path_type_name
            );
            #[cfg(feature = "into_json")]
            if should_check_json {
                check_json(
                    extra_typed_path_type_name,
                    invalid_extra_typed_path_response,
                )
                .await;
            }
            println!("All {} tests passed.", extra_typed_path_type_name);
            Ok(())
        }

        test_extra_typed_path(&test_executor, "extra_typed_path", &server_url).await?;
        test_extra_typed_path_modified(&test_executor, "extra_typed_path_modified", &server_url)
            .await?;
        test_extra_typed_path(
            &test_executor,
            "extra_typed_path_validified_by_ref",
            &server_url,
        )
        .await?;
    }

    #[cfg(feature = "extra_query")]
    {
        use axum_extra::extract::Query;
        test_executor
            .execute::<Query<ParametersValidify>>(Method::POST, extra_query::route::EXTRA_QUERY)
            .await?;
    }

    #[cfg(feature = "extra_form")]
    {
        use axum_extra::extract::Form;
        test_executor
            .execute::<Form<ParametersValidify>>(Method::POST, extra_form::route::EXTRA_FORM)
            .await?;
    }

    #[cfg(feature = "extra_protobuf")]
    {
        use axum_extra::protobuf::Protobuf;
        // Validated
        test_executor
            .execute::<Protobuf<ParametersValidify>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<Protobuf<ParametersValidify>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Protobuf<ParametersValidify>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "yaml")]
    {
        use axum_yaml::Yaml;
        test_executor
            .execute::<Yaml<ParametersValidify>>(Method::POST, yaml::route::YAML)
            .await?;
    }

    #[cfg(feature = "msgpack")]
    {
        use axum_msgpack::{MsgPack, MsgPackRaw};
        test_executor
            .execute::<MsgPack<ParametersValidify>>(Method::POST, msgpack::route::MSGPACK)
            .await?;
        test_executor
            .execute::<MsgPackRaw<ParametersValidify>>(Method::POST, msgpack::route::MSGPACK_RAW)
            .await?;
    }

    drop(server_guard);
    server_handle.await??;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct TestExecutor {
    client: reqwest::Client,
    server_url: Url,
}

impl From<Url> for TestExecutor {
    fn from(server_url: Url) -> Self {
        Self {
            client: Default::default(),
            server_url,
        }
    }
}

impl TestExecutor {
    /// Execute all tests
    pub async fn execute<T: ValidTest>(&self, method: Method, route: &str) -> anyhow::Result<()> {
        self.do_execute::<T>(
            method,
            route,
            StatusCode::OK,
            T::ERROR_STATUS_CODE,
            T::INVALID_STATUS_CODE,
            true,
        )
        .await
    }

    /// Execute all tests for `Modified` without validation
    pub async fn execute_modified<T: ValidTest>(
        &self,
        method: Method,
        route: &str,
    ) -> anyhow::Result<()> {
        self.do_execute::<T>(
            method,
            route,
            StatusCode::OK,
            T::ERROR_STATUS_CODE,
            StatusCode::OK,
            false,
        )
        .await
    }

    /// Execute all tests for `Modified` without validation
    pub async fn execute_validified<T: ValidTest>(
        &self,
        method: Method,
        route: &str,
    ) -> anyhow::Result<()> {
        self.do_execute::<T>(
            method,
            route,
            StatusCode::OK,
            T::INVALID_STATUS_CODE,
            T::INVALID_STATUS_CODE,
            false,
        )
        .await
    }

    async fn do_execute<T: ValidTest>(
        &self,
        method: Method,
        route: &str,
        expected_valid_status: StatusCode,
        expected_error_status: StatusCode,
        expected_invalid_status: StatusCode,
        should_check_json: bool,
    ) -> anyhow::Result<()> {
        let url = {
            let mut url_builder = self.server_url.clone();
            url_builder.set_path(route);
            url_builder
        };

        let type_name = type_name::<T>();

        let valid_builder = self.client.request(method.clone(), url.clone());
        let valid_response = T::set_valid_request(valid_builder).send().await?;
        assert_eq!(
            valid_response.status(),
            expected_valid_status,
            "Validified '{}' test failed: {}.",
            type_name,
            valid_response.text().await?
        );

        let error_builder = self.client.request(method.clone(), url.clone());
        let error_response = T::set_error_request(error_builder).send().await?;
        assert_eq!(
            error_response.status(),
            expected_error_status,
            "Error '{}' test failed: {}.",
            type_name,
            error_response.text().await?
        );

        let invalid_builder = self.client.request(method, url);
        let invalid_response = T::set_invalid_request(invalid_builder).send().await?;
        assert_eq!(
            invalid_response.status(),
            expected_invalid_status,
            "Invalid '{}' test failed: {}.",
            type_name,
            invalid_response.text().await?
        );
        #[cfg(feature = "into_json")]
        if should_check_json && T::JSON_SERIALIZABLE {
            check_json(type_name, invalid_response).await;
        }

        println!("All '{}' tests passed.", type_name);
        Ok(())
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

/// Check if the response is a json response
#[cfg(feature = "into_json")]
pub async fn check_json(type_name: &'static str, response: reqwest::Response) {
    assert_eq!(
        response.headers()[axum::http::header::CONTENT_TYPE],
        axum::http::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        "'{}' rejection into json test failed",
        type_name
    );
    assert!(response.json::<serde_json::Value>().await.is_ok());
}

mod route {
    pub const PATH: &str = "/path/:v0/:v1";
    pub const PATH_MODIFIED: &str = "/path_modified/:v0/:v1";
    pub const PATH_VALIDIFIED: &str = "/path_validified/:v0/:v1";
    pub const PATH_VALIDIFIED_BY_REF: &str = "/path_validified_by_ref/:v0/:v1";
    pub const QUERY: &str = "/query";
    pub const QUERY_MODIFIED: &str = "/query_modified/:v0/:v1";

    pub const QUERY_VALIDIFIED: &str = "/query_validified/:v0/:v1";
    pub const QUERY_VALIDIFIED_BY_REF: &str = "/query_validified_by_ref/:v0/:v1";
    pub const FORM: &str = "/form";
    pub const FORM_MODIFIED: &str = "/form_modified/:v0/:v1";
    pub const FORM_VALIDIFIED: &str = "/form_validified/:v0/:v1";
    pub const FORM_VALIDIFIED_BY_REF: &str = "/form_validified_by_ref/:v0/:v1";
    pub const JSON: &str = "/json";
    pub const JSON_MODIFIED: &str = "/json_modified/:v0/:v1";
    pub const JSON_VALIDIFIED: &str = "/json_validified/:v0/:v1";
    pub const JSON_VALIDIFIED_BY_REF: &str = "/json_validified_by_ref/:v0/:v1";
}

async fn extract_path(
    Validated(Path(parameters)): Validated<Path<ParametersValidify>>,
) -> StatusCode {
    check_validated(&parameters)
}

async fn extract_path_modified(
    Modified(Path(parameters)): Modified<Path<ParametersValidify>>,
) -> StatusCode {
    check_modified(&parameters)
}

async fn extract_path_validified(
    Validified(Path(parameters)): Validified<Path<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_path_validified_by_ref(
    ValidifiedByRef(Path(parameters)): ValidifiedByRef<Path<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_query(
    Validated(Query(parameters)): Validated<Query<ParametersValidify>>,
) -> StatusCode {
    check_validated(&parameters)
}

async fn extract_query_modified(
    Modified(Query(parameters)): Modified<Query<ParametersValidify>>,
) -> StatusCode {
    check_modified(&parameters)
}

async fn extract_query_validified(
    Validified(Query(parameters)): Validified<Query<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_query_validified_by_ref(
    ValidifiedByRef(Query(parameters)): ValidifiedByRef<Query<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_form(
    Validated(Form(parameters)): Validated<Form<ParametersValidify>>,
) -> StatusCode {
    check_validated(&parameters)
}

async fn extract_form_modified(
    Modified(Form(parameters)): Modified<Form<ParametersValidify>>,
) -> StatusCode {
    check_modified(&parameters)
}

async fn extract_form_validified(
    Validified(Form(parameters)): Validified<Form<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_form_validified_by_ref(
    ValidifiedByRef(Form(parameters)): ValidifiedByRef<Form<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_json(
    Validated(Json(parameters)): Validated<Json<ParametersValidify>>,
) -> StatusCode {
    check_validated(&parameters)
}

async fn extract_json_modified(
    Modified(Json(parameters)): Modified<Json<ParametersValidify>>,
) -> StatusCode {
    check_modified(&parameters)
}

async fn extract_json_validified(
    Validified(Json(parameters)): Validified<Json<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

async fn extract_json_validified_by_ref(
    ValidifiedByRef(Json(parameters)): ValidifiedByRef<Json<ParametersValidify>>,
) -> StatusCode {
    check_validified(&parameters)
}

fn check_validated<V: Validate>(validate: &V) -> StatusCode {
    // The `Validified` extractor has validated the `parameters` once,
    // it should have returned `400 BAD REQUEST` if the `parameters` were invalid,
    // Let's validate them again to check if the `Validated` extractor works well.
    // If it works properly, this function will never return `500 INTERNAL SERVER ERROR`
    match validate.validate() {
        Ok(_) => StatusCode::OK,
        Err(e) => {
            eprintln!("Data is unvalidated: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn check_modified<M: IsModified>(modify: &M) -> StatusCode {
    if modify.modified() {
        StatusCode::OK
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    }
}

fn check_validified<D: IsModified + Validate>(data: &D) -> StatusCode {
    let status = check_modified(data);
    if status != StatusCode::OK {
        return status;
    }

    check_validated(data)
}

#[cfg(feature = "typed_header")]
mod typed_header {

    pub(crate) mod route {
        pub const TYPED_HEADER: &str = "/typed_header";
        pub const TYPED_HEADER_MODIFIED: &str = "/typed_header_modified";
        pub const TYPED_HEADER_VALIDIFIED_BY_REF: &str = "/typed_header_validified_be_ref";
    }

    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, ValidifiedByRef};
    use axum::headers::{Error, Header, HeaderName, HeaderValue};
    use axum::http::StatusCode;
    use axum::TypedHeader;

    pub static AXUM_VALID_PARAMETERS: HeaderName = HeaderName::from_static("axum-valid-parameters");

    pub(super) async fn extract_typed_header(
        Validated(TypedHeader(parameters)): Validated<TypedHeader<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub(super) async fn extract_typed_header_modified(
        Modified(TypedHeader(parameters)): Modified<TypedHeader<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub(super) async fn extract_typed_header_validified_by_ref(
        ValidifiedByRef(TypedHeader(parameters)): ValidifiedByRef<TypedHeader<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    impl Header for ParametersValidify {
        fn name() -> &'static HeaderName {
            &AXUM_VALID_PARAMETERS
        }

        fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
        where
            Self: Sized,
            I: Iterator<Item = &'i HeaderValue>,
        {
            let value = values.next().ok_or_else(Error::invalid)?;
            let src = std::str::from_utf8(value.as_bytes()).map_err(|_| Error::invalid())?;
            let split = src.split(',').collect::<Vec<_>>();
            match split.as_slice() {
                [v0, v1] => Ok(ParametersValidify {
                    v0: v0.parse().map_err(|_| Error::invalid())?,
                    v1: v1.to_string(),
                }),
                _ => Err(Error::invalid()),
            }
        }

        fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
            let v0 = self.v0.to_string();
            let mut vec = Vec::with_capacity(v0.len() + 1 + self.v1.len());
            vec.extend_from_slice(v0.as_bytes());
            vec.push(b',');
            vec.extend_from_slice(self.v1.as_bytes());
            let value = HeaderValue::from_bytes(&vec).expect("Failed to build header");
            values.extend(::std::iter::once(value));
        }
    }

    #[test]
    fn parameter_is_header() -> anyhow::Result<()> {
        let parameter = ParametersValidify {
            v0: 123456,
            v1: "111111".to_string(),
        };
        let mut vec = Vec::new();
        parameter.encode(&mut vec);
        let mut iter = vec.iter();
        assert_eq!(parameter, ParametersValidify::decode(&mut iter)?);
        Ok(())
    }
}

#[cfg(feature = "typed_multipart")]
mod typed_multipart {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_typed_multipart::{BaseMultipart, TypedMultipart, TypedMultipartError};

    pub mod route {
        pub const TYPED_MULTIPART: &str = "/typed_multipart";
        pub const TYPED_MULTIPART_MODIFIED: &str = "/typed_multipart_modified";
        pub const TYPED_MULTIPART_VALIDIFIED_BY_REF: &str = "/typed_multipart_validified_by_ref";

        pub const BASE_MULTIPART: &str = "/base_multipart";
        pub const BASE_MULTIPART_MODIFIED: &str = "/base_multipart_modified";
        pub const BASE_MULTIPART_VALIDIFIED_BY_REF: &str = "/base_multipart_validified_by_ref";
    }

    impl From<&ParametersValidify> for reqwest::multipart::Form {
        fn from(value: &ParametersValidify) -> Self {
            reqwest::multipart::Form::new()
                .text("v0", value.v0.to_string())
                .text("v1", value.v1.clone())
        }
    }

    pub(super) async fn extract_typed_multipart(
        Validated(TypedMultipart(parameters)): Validated<TypedMultipart<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub(super) async fn extract_typed_multipart_modified(
        Modified(TypedMultipart(parameters)): Modified<TypedMultipart<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub(super) async fn extract_typed_multipart_validified_by_ref(
        ValidifiedByRef(TypedMultipart(parameters)): ValidifiedByRef<
            TypedMultipart<ParametersValidify>,
        >,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub(super) async fn extract_base_multipart(
        Validated(BaseMultipart { data, .. }): Validated<
            BaseMultipart<ParametersValidify, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_validated(&data)
    }

    pub(super) async fn extract_base_multipart_modified(
        Modified(BaseMultipart { data, .. }): Modified<
            BaseMultipart<ParametersValidify, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_modified(&data)
    }

    pub(super) async fn extract_base_multipart_validified_by_ref(
        ValidifiedByRef(BaseMultipart { data, .. }): ValidifiedByRef<
            BaseMultipart<ParametersValidify, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_validified(&data)
    }
}

#[cfg(feature = "extra")]
mod extra {
    use super::{check_validated, ParametersValidify};
    use crate::tests::{Rejection, ValidTest, ValidTestParameter};
    use crate::{Validated, ValidifyRejection};
    use axum::extract::FromRequestParts;
    use axum::http::request::Parts;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use axum_extra::extract::{Cached, WithRejection};
    use reqwest::RequestBuilder;

    pub mod route {
        pub const CACHED: &str = "/cached";
        pub const WITH_REJECTION: &str = "/with_rejection";
        pub const WITH_REJECTION_VALIDIFY: &str = "/with_rejection_validify";
    }
    pub const PARAMETERS_HEADER: &str = "parameters-header";
    pub const CACHED_REJECTION_STATUS: StatusCode = StatusCode::FORBIDDEN;

    //  1.2. Define you own `Rejection` type and implement `IntoResponse` for it.
    pub enum ParametersRejection {
        Null,
        InvalidJson(serde_json::error::Error),
    }

    impl IntoResponse for ParametersRejection {
        fn into_response(self) -> Response {
            match self {
                ParametersRejection::Null => {
                    (CACHED_REJECTION_STATUS, "My-Data header is missing").into_response()
                }
                ParametersRejection::InvalidJson(e) => (
                    CACHED_REJECTION_STATUS,
                    format!("My-Data is not valid json string: {e}"),
                )
                    .into_response(),
            }
        }
    }

    //  1.3. Implement your extractor (`FromRequestParts` or `FromRequest`)
    #[axum::async_trait]
    impl<S> FromRequestParts<S> for ParametersValidify
    where
        S: Send + Sync,
    {
        type Rejection = ParametersRejection;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
            let Some(value) = parts.headers.get(PARAMETERS_HEADER) else {
                return Err(ParametersRejection::Null);
            };

            serde_json::from_slice(value.as_bytes()).map_err(ParametersRejection::InvalidJson)
        }
    }

    impl ValidTest for ParametersValidify {
        const ERROR_STATUS_CODE: StatusCode = CACHED_REJECTION_STATUS;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersValidify::valid())
                    .expect("Failed to serialize parameters"),
            )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersValidify::error())
                    .expect("Failed to serialize parameters"),
            )
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersValidify::invalid())
                    .expect("Failed to serialize parameters"),
            )
        }
    }

    pub struct ValidifyWithRejectionRejection {
        inner: ParametersRejection,
    }

    impl Rejection for ValidifyWithRejectionRejection {
        const STATUS_CODE: StatusCode = StatusCode::CONFLICT;
    }

    impl IntoResponse for ValidifyWithRejectionRejection {
        fn into_response(self) -> Response {
            let mut response = self.inner.into_response();
            *response.status_mut() = Self::STATUS_CODE;
            response
        }
    }

    // satisfy the `WithRejection`'s extractor trait bound
    // R: From<E::Rejection> + IntoResponse
    impl From<ParametersRejection> for ValidifyWithRejectionRejection {
        fn from(inner: ParametersRejection) -> Self {
            Self { inner }
        }
    }

    pub async fn extract_cached(
        Validated(Cached(parameters)): Validated<Cached<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_with_rejection(
        Validated(WithRejection(parameters, _)): Validated<
            WithRejection<ParametersValidify, ValidifyWithRejectionRejection>,
        >,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub struct WithRejectionValidifyRejection<E> {
        inner: ValidifyRejection<E>,
    }

    impl<E> From<ValidifyRejection<E>> for WithRejectionValidifyRejection<E> {
        fn from(inner: ValidifyRejection<E>) -> Self {
            Self { inner }
        }
    }

    impl<E: IntoResponse> IntoResponse for WithRejectionValidifyRejection<E> {
        fn into_response(self) -> Response {
            let mut res = self.inner.into_response();
            *res.status_mut() = StatusCode::IM_A_TEAPOT;
            res
        }
    }

    pub async fn extract_with_rejection_valid(
        WithRejection(Validated(parameters), _): WithRejection<
            Validated<ParametersValidify>,
            WithRejectionValidifyRejection<ParametersRejection>,
        >,
    ) -> StatusCode {
        check_validated(&parameters)
    }
}

#[cfg(feature = "extra_typed_path")]
mod extra_typed_path {
    use super::{check_modified, check_validated, check_validified};
    use crate::{HasModify, HasValidate, Modified, Validated, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_extra::routing::TypedPath;
    use serde::Deserialize;
    use validify::{Validate, Validify};

    pub mod route {
        pub const EXTRA_TYPED_PATH: &str = "/extra_typed_path/:v0/:v1";
        pub const EXTRA_TYPED_PATH_MODIFIED: &str = "/extra_typed_path_modified/:v0/:v1";
        pub const EXTRA_TYPED_PATH_VALIDIFIED_BY_REF: &str =
            "/extra_typed_path_validified_by_ref/:v0/:v1";
    }

    #[derive(Validate, TypedPath, Deserialize)]
    #[typed_path("/extra_typed_path/:v0/:v1")]
    pub struct TypedPathParam {
        #[validate(range(min = 5.0, max = 10.0))]
        v0: i32,
        #[validate(length(min = 1, max = 10))]
        v1: String,
    }

    impl HasValidate for TypedPathParam {
        type Validate = Self;

        fn get_validate(&self) -> &Self::Validate {
            self
        }
    }

    #[derive(Validify, TypedPath, Deserialize, Clone, PartialEq, Eq)]
    #[typed_path("/extra_typed_path_validified_by_ref/:v0/:v1")]
    pub struct TypedPathParamValidifiedByRef {
        #[validate(range(min = 5.0, max = 10.0))]
        v0: i32,
        #[modify(lowercase)]
        #[validate(length(min = 1, max = 10))]
        v1: String,
    }

    impl HasValidate for TypedPathParamValidifiedByRef {
        type Validate = Self;

        fn get_validate(&self) -> &Self::Validate {
            self
        }
    }

    impl HasModify for TypedPathParamValidifiedByRef {
        type Modify = Self;

        fn get_modify(&mut self) -> &mut Self::Modify {
            self
        }
    }

    #[derive(Validify, TypedPath, Deserialize, Clone, PartialEq, Eq)]
    #[typed_path("/extra_typed_path_modified/:v0/:v1")]
    pub struct TypedPathParamModified {
        #[validate(range(min = 5.0, max = 10.0))]
        v0: i32,
        #[modify(lowercase)]
        #[validate(length(min = 1, max = 10))]
        v1: String,
    }

    impl HasModify for TypedPathParamModified {
        type Modify = Self;

        fn get_modify(&mut self) -> &mut Self::Modify {
            self
        }
    }

    pub async fn extract_extra_typed_path(
        Validated(param): Validated<TypedPathParam>,
    ) -> StatusCode {
        check_validated(&param)
    }

    pub async fn extract_extra_typed_path_modified(
        Modified(param): Modified<TypedPathParamModified>,
    ) -> StatusCode {
        check_modified(&param)
    }

    pub async fn extract_extra_typed_path_validified_by_ref(
        ValidifiedByRef(param): ValidifiedByRef<TypedPathParamValidifiedByRef>,
    ) -> StatusCode {
        check_validified(&param)
    }
}

#[cfg(feature = "extra_query")]
mod extra_query {
    use super::{check_validated, ParametersValidify};
    use crate::Validated;
    use axum::http::StatusCode;
    use axum_extra::extract::Query;

    pub mod route {
        pub const EXTRA_QUERY: &str = "/extra_query";
    }

    pub async fn extract_extra_query(
        Validated(Query(parameters)): Validated<Query<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }
}

#[cfg(feature = "extra_form")]
mod extra_form {
    use super::{check_validated, ParametersValidify};
    use crate::Validated;
    use axum::http::StatusCode;
    use axum_extra::extract::Form;

    pub mod route {
        pub const EXTRA_FORM: &str = "/extra_form";
    }

    pub async fn extract_extra_form(
        Validated(Form(parameters)): Validated<Form<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }
}

#[cfg(feature = "extra_protobuf")]
mod extra_protobuf {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_extra::protobuf::Protobuf;

    pub mod route {
        pub const EXTRA_PROTOBUF: &str = "/extra_protobuf";
        pub const EXTRA_PROTOBUF_MODIFIED: &str = "/extra_protobuf_modified";
        pub const EXTRA_PROTOBUF_VALIDIFIED_BY_REF: &str = "/extra_protobuf_validified_by_ref";
    }

    pub async fn extract_extra_protobuf(
        Validated(Protobuf(parameters)): Validated<Protobuf<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_extra_protobuf_modified(
        Modified(Protobuf(parameters)): Modified<Protobuf<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_extra_protobuf_validified_by_ref(
        ValidifiedByRef(Protobuf(parameters)): ValidifiedByRef<Protobuf<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::{check_validated, ParametersValidify};
    use crate::Validated;
    use axum::http::StatusCode;
    use axum_yaml::Yaml;

    pub mod route {
        pub const YAML: &str = "/yaml";
    }

    pub async fn extract_yaml(
        Validated(Yaml(parameters)): Validated<Yaml<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }
}

#[cfg(feature = "msgpack")]
mod msgpack {
    use super::{check_validated, ParametersValidify};
    use crate::Validated;
    use axum::http::StatusCode;
    use axum_msgpack::{MsgPack, MsgPackRaw};

    pub mod route {
        pub const MSGPACK: &str = "/msgpack";
        pub const MSGPACK_RAW: &str = "/msgpack_raw";
    }

    pub async fn extract_msgpack(
        Validated(MsgPack(parameters)): Validated<MsgPack<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }
    pub async fn extract_msgpack_raw(
        Validated(MsgPackRaw(parameters)): Validated<MsgPackRaw<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }
}
