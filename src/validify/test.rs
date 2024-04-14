#![cfg(feature = "validify")]

use crate::tests::{ValidTest, ValidTestParameter};
use crate::{
    HasValidate, Modified, Validated, Validified, ValidifiedByRef, VALIDATION_ERROR_STATUS,
};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use once_cell::sync::Lazy;
#[cfg(feature = "extra_protobuf")]
use prost::Message;
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::net::TcpListener;
use validify::{Modify, Payload, Validate, Validify};

#[derive(Debug, Clone, Deserialize, Serialize, Validify, Payload, Eq, PartialEq)]
pub struct ParametersValidify {
    #[validate(range(min = 5.0, max = 10.0))]
    v0: i32,
    #[modify(lowercase)]
    #[validate(length(min = 1, max = 10))]
    v1: String,
}

#[derive(Clone, Validify, Eq, PartialEq)]
#[cfg_attr(feature = "extra_protobuf", derive(Message))]
#[cfg_attr(
    feature = "typed_multipart",
    derive(axum_typed_multipart::TryFromMultipart)
)]
pub struct ParametersValidifyWithoutPayload {
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

static VALID_PARAMETERS_WITHOUT_PAYLOAD: Lazy<ParametersValidifyWithoutPayload> =
    Lazy::new(|| ParametersValidifyWithoutPayload {
        v0: 5,
        v1: String::from("ABCDEFG"),
    });

static INVALID_PARAMETERS_WITHOUT_PAYLOAD: Lazy<ParametersValidifyWithoutPayload> =
    Lazy::new(|| ParametersValidifyWithoutPayload {
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

impl ValidTestParameter for ParametersValidifyWithoutPayload {
    fn valid() -> &'static Self {
        VALID_PARAMETERS_WITHOUT_PAYLOAD.deref()
    }

    fn error() -> &'static [(&'static str, &'static str)] {
        &[("not_v0_or_v1", "value")]
    }

    fn invalid() -> &'static Self {
        INVALID_PARAMETERS_WITHOUT_PAYLOAD.deref()
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
            extra::route::CACHED_MODIFIED,
            post(extra::extract_cached_modified),
        )
        .route(
            extra::route::CACHED_VALIDIFIED_BY_REF,
            post(extra::extract_cached_validified_by_ref),
        )
        .route(
            extra::route::WITH_REJECTION,
            post(extra::extract_with_rejection),
        )
        .route(
            extra::route::WITH_REJECTION_MODIFIED,
            post(extra::extract_with_rejection_modified),
        )
        .route(
            extra::route::WITH_REJECTION_VALIDIFIED_BY_REF,
            post(extra::extract_with_rejection_validified_by_ref),
        )
        .route(
            extra::route::WITH_REJECTION_VALIDIFY,
            post(extra::extract_with_rejection_validifiy),
        )
        .route(
            extra::route::WITH_REJECTION_VALIDIFY_MODIFIED,
            post(extra::extract_with_rejection_validifiy_modified),
        )
        .route(
            extra::route::WITH_REJECTION_VALIDIFY_VALIDIFIED_BY_REF,
            post(extra::extract_with_rejection_validifiy_validified_by_ref),
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
    let router = router
        .route(
            extra_query::route::EXTRA_QUERY,
            post(extra_query::extract_extra_query),
        )
        .route(
            extra_query::route::EXTRA_QUERY_MODIFIED,
            post(extra_query::extract_extra_query_modified),
        )
        .route(
            extra_query::route::EXTRA_QUERY_VALIDIFIED,
            post(extra_query::extract_extra_query_validified),
        )
        .route(
            extra_query::route::EXTRA_QUERY_VALIDIFIED_BY_REF,
            post(extra_query::extract_extra_query_validified_by_ref),
        );

    #[cfg(feature = "extra_form")]
    let router = router
        .route(
            extra_form::route::EXTRA_FORM,
            post(extra_form::extract_extra_form),
        )
        .route(
            extra_form::route::EXTRA_FORM_MODIFIED,
            post(extra_form::extract_extra_form_modified),
        )
        .route(
            extra_form::route::EXTRA_FORM_VALIDIFED,
            post(extra_form::extract_extra_form_validifed),
        )
        .route(
            extra_form::route::EXTRA_FORM_VALIDIFED_BY_REF,
            post(extra_form::extract_extra_form_validifed_by_ref),
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
    let router = router
        .route(yaml::route::YAML, post(yaml::extract_yaml))
        .route(
            yaml::route::YAML_MODIFIED,
            post(yaml::extract_yaml_modified),
        )
        .route(
            yaml::route::YAML_VALIDIFIED,
            post(yaml::extract_yaml_validified),
        )
        .route(
            yaml::route::YAML_VALIDIFIED_BY_REF,
            post(yaml::extract_yaml_validified_by_ref),
        );

    #[cfg(feature = "msgpack")]
    let router = router
        .route(msgpack::route::MSGPACK, post(msgpack::extract_msgpack))
        .route(
            msgpack::route::MSGPACK_MODIFIED,
            post(msgpack::extract_msgpack_modified),
        )
        .route(
            msgpack::route::MSGPACK_VALIDIFIED,
            post(msgpack::extract_msgpack_validified),
        )
        .route(
            msgpack::route::MSGPACK_VALIDIFIED_BY_REF,
            post(msgpack::extract_msgpack_validified_by_ref),
        )
        .route(
            msgpack::route::MSGPACK_RAW,
            post(msgpack::extract_msgpack_raw),
        )
        .route(
            msgpack::route::MSGPACK_RAW_MODIFIED,
            post(msgpack::extract_msgpack_raw_modified),
        )
        .route(
            msgpack::route::MSGPACK_RAW_VALIDIFIED,
            post(msgpack::extract_msgpack_raw_validified),
        )
        .route(
            msgpack::route::MSGPACK_RAW_VALIDIFIED_BY_REF,
            post(msgpack::extract_msgpack_raw_validified_by_ref),
        );

    #[cfg(feature = "xml")]
    let router = router
        .route(xml::route::XML, post(xml::extract_xml))
        .route(xml::route::XML_MODIFIED, post(xml::extract_xml_modified))
        .route(
            xml::route::XML_VALIDIFIED,
            post(xml::extract_xml_validified),
        )
        .route(
            xml::route::XML_VALIDIFIED_BY_REF,
            post(xml::extract_xml_validified_by_ref),
        );

    #[cfg(feature = "toml")]
    let router = router
        .route(toml::route::TOML, post(toml::extract_toml))
        .route(
            toml::route::TOML_MODIFIED,
            post(toml::extract_toml_modified),
        )
        .route(
            toml::route::TOML_VALIDIFIED,
            post(toml::extract_toml_validified),
        )
        .route(
            toml::route::TOML_VALIDIFIED_BY_REF,
            post(toml::extract_toml_validified_by_ref),
        );

    #[cfg(feature = "sonic")]
    let router = router
        .route(sonic::route::SONIC, post(sonic::extract_sonic))
        .route(
            sonic::route::SONIC_MODIFIED,
            post(sonic::extract_sonic_modified),
        )
        .route(
            sonic::route::SONIC_VALIDIFIED,
            post(sonic::extract_sonic_validified),
        )
        .route(
            sonic::route::SONIC_VALIDIFIED_BY_REF,
            post(sonic::extract_sonic_validified_by_ref),
        );

    #[cfg(feature = "cbor")]
    let router = router
        .route(cbor::route::CBOR, post(cbor::extract_cbor))
        .route(
            cbor::route::CBOR_MODIFIED,
            post(cbor::extract_cbor_modified),
        )
        .route(
            cbor::route::CBOR_VALIDIFIED,
            post(cbor::extract_cbor_validified),
        )
        .route(
            cbor::route::CBOR_VALIDIFIED_BY_REF,
            post(cbor::extract_cbor_validified_by_ref),
        );

    let listener = TcpListener::bind(&SocketAddr::from(([0u8, 0, 0, 0], 0u16))).await?;
    let server_addr = listener.local_addr()?;
    let server = axum::serve(listener, router.into_make_service());
    println!("Axum server address: {}.", server_addr);

    tokio::spawn(async move {
        let _ = server.await;
    });

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
            valid_path_response.status().as_u16(),
            expected_valid_status.as_u16(),
            "Valid '{}' test failed.",
            path_type_name
        );

        let error_path_response = test_executor
            .client()
            .get(format!("{}/{route}/not_i32/path", server_url))
            .send()
            .await?;
        assert_eq!(
            error_path_response.status().as_u16(),
            expected_error_status.as_u16(),
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
            invalid_path_response.status().as_u16(),
            expected_invalid_status.as_u16(),
            "Invalid '{}' test failed.",
            path_type_name
        );
        if should_check_json {
            #[cfg(feature = "into_json")]
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
        use axum_extra::typed_header::TypedHeader;
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
        // Validated
        test_executor
            .execute::<Cached<ParametersValidify>>(Method::POST, extra::route::CACHED)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Cached<ParametersValidify>>(
                Method::POST,
                extra::route::CACHED_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Cached<ParametersValidify>>(
                Method::POST,
                extra::route::CACHED_VALIDIFIED_BY_REF,
            )
            .await?;

        // Validated
        test_executor
            .execute::<WithRejection<ParametersValidify, ValidifyWithRejectionRejection>>(
                Method::POST,
                extra::route::WITH_REJECTION,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<WithRejection<ParametersValidify, ValidifyWithRejectionRejection>>(
                Method::POST,
                extra::route::WITH_REJECTION_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<WithRejection<ParametersValidify, ValidifyWithRejectionRejection>>(
                Method::POST,
                extra::route::WITH_REJECTION_VALIDIFIED_BY_REF,
            )
            .await?;

        // Validated
        test_executor
            .execute::<WithRejection<
                Validated<ParametersValidify>,
                WithRejectionValidifyRejection<ParametersRejection>,
            >>(Method::POST, extra::route::WITH_REJECTION_VALIDIFY)
            .await?;
        // Modified
        test_executor
            .execute_modified::<WithRejection<
                Modified<ParametersValidify>,
                ValidifyWithRejectionRejection,
            >>(Method::POST, extra::route::WITH_REJECTION_VALIDIFY_MODIFIED)
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<WithRejection<
                ValidifiedByRef<ParametersValidify>,
                WithRejectionValidifyRejection<ParametersRejection>,
            >>(
                Method::POST,
                extra::route::WITH_REJECTION_VALIDIFY_VALIDIFIED_BY_REF,
            )
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
                valid_extra_typed_path_response.status().as_u16(),
                expected_valid_status.as_u16(),
                "Validified '{}' test failed.",
                extra_typed_path_type_name
            );

            let error_extra_typed_path_response = test_executor
                .client()
                .get(format!("{}/{route}/not_i32/path", server_url))
                .send()
                .await?;
            assert_eq!(
                error_extra_typed_path_response.status().as_u16(),
                expected_error_status.as_u16(),
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
                invalid_extra_typed_path_response.status().as_u16(),
                expected_invalid_status.as_u16(),
                "Invalid '{}' test failed.",
                extra_typed_path_type_name
            );

            if should_check_json {
                #[cfg(feature = "into_json")]
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
        // Validated
        test_executor
            .execute::<Query<ParametersValidify>>(Method::POST, extra_query::route::EXTRA_QUERY)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Query<ParametersValidify>>(
                Method::POST,
                extra_query::route::EXTRA_QUERY_MODIFIED,
            )
            .await?;
        // Validified
        test_executor
            .execute_validified::<Query<ParametersValidify>>(
                Method::POST,
                extra_query::route::EXTRA_QUERY_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Query<ParametersValidify>>(
                Method::POST,
                extra_query::route::EXTRA_QUERY_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "extra_form")]
    {
        use axum_extra::extract::Form;
        // Validated
        test_executor
            .execute::<Form<ParametersValidify>>(Method::POST, extra_form::route::EXTRA_FORM)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Form<ParametersValidify>>(
                Method::POST,
                extra_form::route::EXTRA_FORM_MODIFIED,
            )
            .await?;
        // Validified
        test_executor
            .execute_validified::<Form<ParametersValidify>>(
                Method::POST,
                extra_form::route::EXTRA_FORM_VALIDIFED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Form<ParametersValidify>>(
                Method::POST,
                extra_form::route::EXTRA_FORM_VALIDIFED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "extra_protobuf")]
    {
        use axum_extra::protobuf::Protobuf;
        // Validated
        test_executor
            .execute::<Protobuf<ParametersValidifyWithoutPayload>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF,
            )
            .await?;
        // Modified
        test_executor
            .execute_modified::<Protobuf<ParametersValidifyWithoutPayload>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF_MODIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Protobuf<ParametersValidifyWithoutPayload>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "yaml")]
    {
        use axum_serde::Yaml;

        // Validated
        test_executor
            .execute::<Yaml<ParametersValidify>>(Method::POST, yaml::route::YAML)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Yaml<ParametersValidify>>(Method::POST, yaml::route::YAML_MODIFIED)
            .await?;
        // Validified
        test_executor
            .execute_validified::<Yaml<ParametersValidify>>(
                Method::POST,
                yaml::route::YAML_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Yaml<ParametersValidify>>(Method::POST, yaml::route::YAML_VALIDIFIED_BY_REF)
            .await?;
    }

    #[cfg(feature = "msgpack")]
    {
        use axum_serde::{MsgPack, MsgPackRaw};
        // Validated
        test_executor
            .execute::<MsgPack<ParametersValidify>>(Method::POST, msgpack::route::MSGPACK)
            .await?;
        // Modified
        test_executor
            .execute_modified::<MsgPack<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_MODIFIED,
            )
            .await?;
        // Validified
        test_executor
            .execute_validified::<MsgPack<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<MsgPack<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_VALIDIFIED_BY_REF,
            )
            .await?;

        //
        test_executor
            .execute::<MsgPackRaw<ParametersValidify>>(Method::POST, msgpack::route::MSGPACK_RAW)
            .await?;
        //
        test_executor
            .execute_modified::<MsgPackRaw<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_RAW_MODIFIED,
            )
            .await?;
        //
        test_executor
            .execute_validified::<MsgPackRaw<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_RAW_VALIDIFIED,
            )
            .await?;
        //
        test_executor
            .execute::<MsgPackRaw<ParametersValidify>>(
                Method::POST,
                msgpack::route::MSGPACK_RAW_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "xml")]
    {
        use axum_serde::Xml;

        // Validated
        test_executor
            .execute::<Xml<ParametersValidify>>(Method::POST, xml::route::XML)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Xml<ParametersValidify>>(Method::POST, xml::route::XML_MODIFIED)
            .await?;
        // Validified
        test_executor
            .execute_validified::<Xml<ParametersValidify>>(Method::POST, xml::route::XML_VALIDIFIED)
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Xml<ParametersValidify>>(Method::POST, xml::route::XML_VALIDIFIED_BY_REF)
            .await?;
    }

    #[cfg(feature = "toml")]
    {
        use axum_serde::Toml;

        // Validated
        test_executor
            .execute::<Toml<ParametersValidify>>(Method::POST, toml::route::TOML)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Toml<ParametersValidify>>(Method::POST, toml::route::TOML_MODIFIED)
            .await?;
        // Validified
        test_executor
            .execute_validified::<Toml<ParametersValidify>>(
                Method::POST,
                toml::route::TOML_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Toml<ParametersValidify>>(Method::POST, toml::route::TOML_VALIDIFIED_BY_REF)
            .await?;
    }

    #[cfg(feature = "sonic")]
    {
        use axum_serde::Sonic;

        // Validated
        test_executor
            .execute::<Sonic<ParametersValidify>>(Method::POST, sonic::route::SONIC)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Sonic<ParametersValidify>>(
                Method::POST,
                sonic::route::SONIC_MODIFIED,
            )
            .await?;
        // Validified
        test_executor
            .execute_validified::<Sonic<ParametersValidify>>(
                Method::POST,
                sonic::route::SONIC_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Sonic<ParametersValidify>>(
                Method::POST,
                sonic::route::SONIC_VALIDIFIED_BY_REF,
            )
            .await?;
    }

    #[cfg(feature = "cbor")]
    {
        use axum_serde::Cbor;

        // Validated
        test_executor
            .execute::<Cbor<ParametersValidify>>(Method::POST, cbor::route::CBOR)
            .await?;
        // Modified
        test_executor
            .execute_modified::<Cbor<ParametersValidify>>(Method::POST, cbor::route::CBOR_MODIFIED)
            .await?;
        // Validified
        test_executor
            .execute_validified::<Cbor<ParametersValidify>>(
                Method::POST,
                cbor::route::CBOR_VALIDIFIED,
            )
            .await?;
        // ValidifiedByRef
        test_executor
            .execute::<Cbor<ParametersValidify>>(Method::POST, cbor::route::CBOR_VALIDIFIED_BY_REF)
            .await?;
    }

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
            valid_response.status().as_u16(),
            expected_valid_status.as_u16(),
            "Validified '{}' test failed: {}.",
            type_name,
            valid_response.text().await?
        );

        let error_builder = self.client.request(method.clone(), url.clone());
        let error_response = T::set_error_request(error_builder).send().await?;
        assert_eq!(
            error_response.status().as_u16(),
            expected_error_status.as_u16(),
            "Error '{}' test failed: {}.",
            type_name,
            error_response.text().await?
        );

        let invalid_builder = self.client.request(method, url);
        let invalid_response = T::set_invalid_request(invalid_builder).send().await?;
        assert_eq!(
            invalid_response.status().as_u16(),
            expected_invalid_status.as_u16(),
            "Invalid '{}' test failed: {}.",
            type_name,
            invalid_response.text().await?
        );
        if should_check_json {
            #[cfg(feature = "into_json")]
            if T::JSON_SERIALIZABLE {
                check_json(type_name, invalid_response).await;
            }
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
        response.headers()[reqwest::header::CONTENT_TYPE],
        reqwest::header::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
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
    use axum::http::StatusCode;
    use axum_extra::headers::{Error, Header, HeaderName, HeaderValue};
    use axum_extra::typed_header::TypedHeader;

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
    use super::{
        check_modified, check_validated, check_validified, ParametersValidify,
        ParametersValidifyWithoutPayload,
    };
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
        Validated(TypedMultipart(parameters)): Validated<
            TypedMultipart<ParametersValidifyWithoutPayload>,
        >,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub(super) async fn extract_typed_multipart_modified(
        Modified(TypedMultipart(parameters)): Modified<
            TypedMultipart<ParametersValidifyWithoutPayload>,
        >,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub(super) async fn extract_typed_multipart_validified_by_ref(
        ValidifiedByRef(TypedMultipart(parameters)): ValidifiedByRef<
            TypedMultipart<ParametersValidifyWithoutPayload>,
        >,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub(super) async fn extract_base_multipart(
        Validated(BaseMultipart { data, .. }): Validated<
            BaseMultipart<ParametersValidifyWithoutPayload, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_validated(&data)
    }

    pub(super) async fn extract_base_multipart_modified(
        Modified(BaseMultipart { data, .. }): Modified<
            BaseMultipart<ParametersValidifyWithoutPayload, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_modified(&data)
    }

    pub(super) async fn extract_base_multipart_validified_by_ref(
        ValidifiedByRef(BaseMultipart { data, .. }): ValidifiedByRef<
            BaseMultipart<ParametersValidifyWithoutPayload, TypedMultipartError>,
        >,
    ) -> StatusCode {
        check_validified(&data)
    }
}

#[cfg(feature = "extra")]
mod extra {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::tests::{Rejection, ValidTest, ValidTestParameter};
    use crate::{HasModify, Modified, Validated, ValidifiedByRef, ValidifyRejection};
    use axum::extract::FromRequestParts;
    use axum::http::request::Parts;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use axum_extra::extract::{Cached, WithRejection};
    use reqwest::RequestBuilder;

    pub mod route {
        pub const CACHED: &str = "/cached";
        pub const CACHED_MODIFIED: &str = "/cached_modified";
        pub const CACHED_VALIDIFIED_BY_REF: &str = "/cached_validified_by_ref";
        pub const WITH_REJECTION: &str = "/with_rejection";
        pub const WITH_REJECTION_MODIFIED: &str = "/with_rejection_modified";
        pub const WITH_REJECTION_VALIDIFIED_BY_REF: &str = "/with_rejection_validified_by_ref";
        pub const WITH_REJECTION_VALIDIFY: &str = "/with_rejection_validify";
        pub const WITH_REJECTION_VALIDIFY_MODIFIED: &str = "/with_rejection_validify_modified";
        pub const WITH_REJECTION_VALIDIFY_VALIDIFIED_BY_REF: &str =
            "/with_rejection_validify_validified_by_ref";
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

    impl HasModify for ParametersValidify {
        type Modify = Self;

        fn get_modify(&mut self) -> &mut Self::Modify {
            self
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

    pub async fn extract_cached_modified(
        Modified(Cached(parameters)): Modified<Cached<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_cached_validified_by_ref(
        ValidifiedByRef(Cached(parameters)): ValidifiedByRef<Cached<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_with_rejection(
        Validated(WithRejection(parameters, _)): Validated<
            WithRejection<ParametersValidify, ValidifyWithRejectionRejection>,
        >,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_with_rejection_modified(
        Modified(WithRejection(parameters, _)): Modified<
            WithRejection<ParametersValidify, ValidifyWithRejectionRejection>,
        >,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_with_rejection_validified_by_ref(
        ValidifiedByRef(WithRejection(parameters, _)): ValidifiedByRef<
            WithRejection<ParametersValidify, ValidifyWithRejectionRejection>,
        >,
    ) -> StatusCode {
        check_validified(&parameters)
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

    pub async fn extract_with_rejection_validifiy(
        WithRejection(Validated(parameters), _): WithRejection<
            Validated<ParametersValidify>,
            WithRejectionValidifyRejection<ParametersRejection>,
        >,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_with_rejection_validifiy_modified(
        WithRejection(Modified(parameters), _): WithRejection<
            Modified<ParametersValidify>,
            ValidifyWithRejectionRejection,
        >,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_with_rejection_validifiy_validified_by_ref(
        WithRejection(ValidifiedByRef(parameters), _): WithRejection<
            ValidifiedByRef<ParametersValidify>,
            WithRejectionValidifyRejection<ParametersRejection>,
        >,
    ) -> StatusCode {
        check_validified(&parameters)
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
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_extra::extract::Query;

    pub mod route {
        pub const EXTRA_QUERY: &str = "/extra_query";
        pub const EXTRA_QUERY_MODIFIED: &str = "/extra_query_modified";
        pub const EXTRA_QUERY_VALIDIFIED: &str = "/extra_query_validified";
        pub const EXTRA_QUERY_VALIDIFIED_BY_REF: &str = "/extra_query_validified_by_ref";
    }

    pub async fn extract_extra_query(
        Validated(Query(parameters)): Validated<Query<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_extra_query_modified(
        Modified(Query(parameters)): Modified<Query<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_extra_query_validified(
        Validified(Query(parameters)): Validified<Query<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_extra_query_validified_by_ref(
        ValidifiedByRef(Query(parameters)): ValidifiedByRef<Query<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "extra_form")]
mod extra_form {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_extra::extract::Form;

    pub mod route {
        pub const EXTRA_FORM: &str = "/extra_form";
        pub const EXTRA_FORM_MODIFIED: &str = "/extra_form_modified";
        pub const EXTRA_FORM_VALIDIFED: &str = "/extra_form_validifed";
        pub const EXTRA_FORM_VALIDIFED_BY_REF: &str = "/extra_form_validified_by_ref";
    }

    pub async fn extract_extra_form(
        Validated(Form(parameters)): Validated<Form<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_extra_form_modified(
        Modified(Form(parameters)): Modified<Form<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_extra_form_validifed(
        Validified(Form(parameters)): Validified<Form<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_extra_form_validifed_by_ref(
        ValidifiedByRef(Form(parameters)): ValidifiedByRef<Form<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "extra_protobuf")]
mod extra_protobuf {
    use super::{
        check_modified, check_validated, check_validified, ParametersValidifyWithoutPayload,
    };
    use crate::{Modified, Validated, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_extra::protobuf::Protobuf;

    pub mod route {
        pub const EXTRA_PROTOBUF: &str = "/extra_protobuf";
        pub const EXTRA_PROTOBUF_MODIFIED: &str = "/extra_protobuf_modified";
        pub const EXTRA_PROTOBUF_VALIDIFIED_BY_REF: &str = "/extra_protobuf_validified_by_ref";
    }

    pub async fn extract_extra_protobuf(
        Validated(Protobuf(parameters)): Validated<Protobuf<ParametersValidifyWithoutPayload>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_extra_protobuf_modified(
        Modified(Protobuf(parameters)): Modified<Protobuf<ParametersValidifyWithoutPayload>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_extra_protobuf_validified_by_ref(
        ValidifiedByRef(Protobuf(parameters)): ValidifiedByRef<
            Protobuf<ParametersValidifyWithoutPayload>,
        >,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::Yaml;

    pub mod route {
        pub const YAML: &str = "/yaml";
        pub const YAML_MODIFIED: &str = "/yaml_modified";
        pub const YAML_VALIDIFIED: &str = "/yaml_validified";
        pub const YAML_VALIDIFIED_BY_REF: &str = "/yaml_validified_by_ref";
    }

    pub async fn extract_yaml(
        Validated(Yaml(parameters)): Validated<Yaml<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_yaml_modified(
        Modified(Yaml(parameters)): Modified<Yaml<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_yaml_validified(
        Validified(Yaml(parameters)): Validified<Yaml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_yaml_validified_by_ref(
        ValidifiedByRef(Yaml(parameters)): ValidifiedByRef<Yaml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "msgpack")]
mod msgpack {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::{MsgPack, MsgPackRaw};

    pub mod route {
        pub const MSGPACK: &str = "/msgpack";
        pub const MSGPACK_MODIFIED: &str = "/msgpack_modified";
        pub const MSGPACK_VALIDIFIED: &str = "/msgpack_validified";
        pub const MSGPACK_VALIDIFIED_BY_REF: &str = "/msgpack_validified_by_ref";
        pub const MSGPACK_RAW: &str = "/msgpack_raw";
        pub const MSGPACK_RAW_MODIFIED: &str = "/msgpack_raw_modified";
        pub const MSGPACK_RAW_VALIDIFIED: &str = "/msgpack_raw_validified";
        pub const MSGPACK_RAW_VALIDIFIED_BY_REF: &str = "/msgpack_raw_validified_by_ref";
    }

    pub async fn extract_msgpack(
        Validated(MsgPack(parameters)): Validated<MsgPack<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_msgpack_modified(
        Modified(MsgPack(parameters)): Modified<MsgPack<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_msgpack_validified(
        Validified(MsgPack(parameters)): Validified<MsgPack<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_msgpack_validified_by_ref(
        ValidifiedByRef(MsgPack(parameters)): ValidifiedByRef<MsgPack<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_msgpack_raw(
        Validated(MsgPackRaw(parameters)): Validated<MsgPackRaw<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_msgpack_raw_modified(
        Modified(MsgPackRaw(parameters)): Modified<MsgPackRaw<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_msgpack_raw_validified(
        Validified(MsgPackRaw(parameters)): Validified<MsgPackRaw<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_msgpack_raw_validified_by_ref(
        ValidifiedByRef(MsgPackRaw(parameters)): ValidifiedByRef<MsgPackRaw<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "xml")]
mod xml {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::Xml;

    pub mod route {
        pub const XML: &str = "/xml";
        pub const XML_MODIFIED: &str = "/xml_modified";
        pub const XML_VALIDIFIED: &str = "/xml_validified";
        pub const XML_VALIDIFIED_BY_REF: &str = "/xml_validified_by_ref";
    }

    pub async fn extract_xml(
        Validated(Xml(parameters)): Validated<Xml<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_xml_modified(
        Modified(Xml(parameters)): Modified<Xml<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_xml_validified(
        Validified(Xml(parameters)): Validified<Xml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_xml_validified_by_ref(
        ValidifiedByRef(Xml(parameters)): ValidifiedByRef<Xml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::Toml;

    pub mod route {
        pub const TOML: &str = "/toml";
        pub const TOML_MODIFIED: &str = "/toml_modified";
        pub const TOML_VALIDIFIED: &str = "/toml_validified";
        pub const TOML_VALIDIFIED_BY_REF: &str = "/toml_validified_by_ref";
    }

    pub async fn extract_toml(
        Validated(Toml(parameters)): Validated<Toml<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_toml_modified(
        Modified(Toml(parameters)): Modified<Toml<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_toml_validified(
        Validified(Toml(parameters)): Validified<Toml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_toml_validified_by_ref(
        ValidifiedByRef(Toml(parameters)): ValidifiedByRef<Toml<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "sonic")]
mod sonic {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::Sonic;

    pub mod route {
        pub const SONIC: &str = "/sonic";
        pub const SONIC_MODIFIED: &str = "/sonic_modified";
        pub const SONIC_VALIDIFIED: &str = "/sonic_validified";
        pub const SONIC_VALIDIFIED_BY_REF: &str = "/sonic_validified_by_ref";
    }

    pub async fn extract_sonic(
        Validated(Sonic(parameters)): Validated<Sonic<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_sonic_modified(
        Modified(Sonic(parameters)): Modified<Sonic<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_sonic_validified(
        Validified(Sonic(parameters)): Validified<Sonic<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_sonic_validified_by_ref(
        ValidifiedByRef(Sonic(parameters)): ValidifiedByRef<Sonic<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}

#[cfg(feature = "cbor")]
mod cbor {
    use super::{check_modified, check_validated, check_validified, ParametersValidify};
    use crate::{Modified, Validated, Validified, ValidifiedByRef};
    use axum::http::StatusCode;
    use axum_serde::Cbor;

    pub mod route {
        pub const CBOR: &str = "/cbor";
        pub const CBOR_MODIFIED: &str = "/cbor_modified";
        pub const CBOR_VALIDIFIED: &str = "/cbor_validified";
        pub const CBOR_VALIDIFIED_BY_REF: &str = "/cbor_validified_by_ref";
    }

    pub async fn extract_cbor(
        Validated(Cbor(parameters)): Validated<Cbor<ParametersValidify>>,
    ) -> StatusCode {
        check_validated(&parameters)
    }

    pub async fn extract_cbor_modified(
        Modified(Cbor(parameters)): Modified<Cbor<ParametersValidify>>,
    ) -> StatusCode {
        check_modified(&parameters)
    }

    pub async fn extract_cbor_validified(
        Validified(Cbor(parameters)): Validified<Cbor<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }

    pub async fn extract_cbor_validified_by_ref(
        ValidifiedByRef(Cbor(parameters)): ValidifiedByRef<Cbor<ParametersValidify>>,
    ) -> StatusCode {
        check_validified(&parameters)
    }
}
