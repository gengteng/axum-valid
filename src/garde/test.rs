#![cfg(feature = "garde")]

use crate::tests::{ValidTest, ValidTestParameter};
use crate::{Garde, HasValidate, VALIDATION_ERROR_STATUS};
use axum::extract::{FromRef, Path, Query};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Form, Json, Router};
use garde::Validate;
use once_cell::sync::Lazy;
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::net::SocketAddr;
use std::ops::Deref;
use tokio::net::TcpListener;

#[derive(Clone, Deserialize, Serialize, Validate, Eq, PartialEq)]
#[cfg_attr(feature = "extra_protobuf", derive(prost::Message))]
#[cfg_attr(
    feature = "typed_multipart",
    derive(axum_typed_multipart::TryFromMultipart)
)]
pub struct ParametersGarde {
    #[garde(range(min = 5, max = 10))]
    #[cfg_attr(feature = "extra_protobuf", prost(int32, tag = "1"))]
    v0: i32,
    #[garde(length(min = 1, max = 10))]
    #[cfg_attr(feature = "extra_protobuf", prost(string, tag = "2"))]
    v1: String,
}

static VALID_PARAMETERS: Lazy<ParametersGarde> = Lazy::new(|| ParametersGarde {
    v0: 5,
    v1: String::from("0123456789"),
});

static INVALID_PARAMETERS: Lazy<ParametersGarde> = Lazy::new(|| ParametersGarde {
    v0: 6,
    v1: String::from("01234567890"),
});

#[derive(Debug, Clone, FromRef, Default)]
struct MyState {
    no_argument_context: (),
}

impl ValidTestParameter for ParametersGarde {
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

impl HasValidate for ParametersGarde {
    type Validate = ParametersGarde;

    fn get_validate(&self) -> &Self::Validate {
        self
    }
}

#[tokio::test]
async fn test_main() -> anyhow::Result<()> {
    let router = Router::new()
        .route(route::PATH, get(extract_path))
        .route(route::QUERY, get(extract_query))
        .route(route::FORM, post(extract_form))
        .route(route::JSON, post(extract_json));

    #[cfg(feature = "typed_header")]
    let router = router.route(
        typed_header::route::TYPED_HEADER,
        post(typed_header::extract_typed_header),
    );

    #[cfg(feature = "typed_multipart")]
    let router = router
        .route(
            typed_multipart::route::TYPED_MULTIPART,
            post(typed_multipart::extract_typed_multipart),
        )
        .route(
            typed_multipart::route::BASE_MULTIPART,
            post(typed_multipart::extract_base_multipart),
        );

    #[cfg(feature = "extra")]
    let router = router
        .route(extra::route::CACHED, post(extra::extract_cached))
        .route(
            extra::route::WITH_REJECTION,
            post(extra::extract_with_rejection),
        )
        .route(
            extra::route::WITH_REJECTION_GARDE,
            post(extra::extract_with_rejection_valid),
        );

    #[cfg(feature = "extra_typed_path")]
    let router = router.route(
        extra_typed_path::route::EXTRA_TYPED_PATH,
        get(extra_typed_path::extract_extra_typed_path),
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
    let router = router.route(
        extra_protobuf::route::EXTRA_PROTOBUF,
        post(extra_protobuf::extract_extra_protobuf),
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

    #[cfg(feature = "xml")]
    let router = router.route(xml::route::XML, post(xml::extract_xml));

    #[cfg(feature = "toml")]
    let router = router.route(toml::route::TOML, post(toml::extract_toml));

    #[cfg(feature = "sonic")]
    let router = router.route(sonic::route::SONIC, post(sonic::extract_sonic));

    #[cfg(feature = "cbor")]
    let router = router.route(cbor::route::CBOR, post(cbor::extract_cbor));

    let router = router.with_state(MyState::default());

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
        let path_type_name = type_name::<Path<ParametersGarde>>();
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
            StatusCode::OK.as_u16(),
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
            StatusCode::BAD_REQUEST.as_u16(),
            "Error '{}' test failed.",
            path_type_name
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
            VALIDATION_ERROR_STATUS.as_u16(),
            "Invalid '{}' test failed.",
            path_type_name
        );
        #[cfg(feature = "into_json")]
        check_json(path_type_name, invalid_path_response).await;
        println!("All {} tests passed.", path_type_name);
        Ok(())
    }

    test_extra_path(&test_executor, "path", &server_url).await?;

    // Garde
    test_executor
        .execute::<Query<ParametersGarde>>(Method::GET, route::QUERY)
        .await?;

    // Garde
    test_executor
        .execute::<Form<ParametersGarde>>(Method::POST, route::FORM)
        .await?;

    // Garde
    test_executor
        .execute::<Json<ParametersGarde>>(Method::POST, route::JSON)
        .await?;

    #[cfg(feature = "typed_header")]
    {
        use axum_extra::typed_header::TypedHeader;
        // Garde
        test_executor
            .execute::<TypedHeader<ParametersGarde>>(
                Method::POST,
                typed_header::route::TYPED_HEADER,
            )
            .await?;
    }

    #[cfg(feature = "typed_multipart")]
    {
        use axum_typed_multipart::{BaseMultipart, TypedMultipart, TypedMultipartError};

        // Garde
        test_executor
            .execute::<BaseMultipart<ParametersGarde, TypedMultipartError>>(
                Method::POST,
                typed_multipart::route::BASE_MULTIPART,
            )
            .await?;

        // Garde
        test_executor
            .execute::<TypedMultipart<ParametersGarde>>(
                Method::POST,
                typed_multipart::route::TYPED_MULTIPART,
            )
            .await?;
    }

    #[cfg(feature = "extra")]
    {
        use axum_extra::extract::{Cached, WithRejection};
        use extra::{
            GardeWithRejectionRejection, ParametersRejection, WithRejectionGardeRejection,
        };
        test_executor
            .execute::<Cached<ParametersGarde>>(Method::POST, extra::route::CACHED)
            .await?;
        test_executor
            .execute::<WithRejection<ParametersGarde, GardeWithRejectionRejection>>(
                Method::POST,
                extra::route::WITH_REJECTION,
            )
            .await?;
        test_executor
            .execute::<WithRejection<Garde<ParametersGarde>, WithRejectionGardeRejection<ParametersRejection>>>(
                Method::POST,
                extra::route::WITH_REJECTION_GARDE,
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
                StatusCode::OK.as_u16(),
                "Garde '{}' test failed.",
                extra_typed_path_type_name
            );

            let error_extra_typed_path_response = test_executor
                .client()
                .get(format!("{}/{route}/not_i32/path", server_url))
                .send()
                .await?;
            assert_eq!(
                error_extra_typed_path_response.status().as_u16(),
                StatusCode::BAD_REQUEST.as_u16(),
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
                VALIDATION_ERROR_STATUS.as_u16(),
                "Invalid '{}' test failed.",
                extra_typed_path_type_name
            );
            #[cfg(feature = "into_json")]
            check_json(
                extra_typed_path_type_name,
                invalid_extra_typed_path_response,
            )
            .await;
            println!("All {} tests passed.", extra_typed_path_type_name);
            Ok(())
        }

        test_extra_typed_path(&test_executor, "extra_typed_path", &server_url).await?;
    }

    #[cfg(feature = "extra_query")]
    {
        use axum_extra::extract::Query;
        test_executor
            .execute::<Query<ParametersGarde>>(Method::POST, extra_query::route::EXTRA_QUERY)
            .await?;
    }

    #[cfg(feature = "extra_form")]
    {
        use axum_extra::extract::Form;
        test_executor
            .execute::<Form<ParametersGarde>>(Method::POST, extra_form::route::EXTRA_FORM)
            .await?;
    }

    #[cfg(feature = "extra_protobuf")]
    {
        use axum_extra::protobuf::Protobuf;
        test_executor
            .execute::<Protobuf<ParametersGarde>>(
                Method::POST,
                extra_protobuf::route::EXTRA_PROTOBUF,
            )
            .await?;
    }

    #[cfg(feature = "yaml")]
    {
        use axum_serde::Yaml;
        test_executor
            .execute::<Yaml<ParametersGarde>>(Method::POST, yaml::route::YAML)
            .await?;
    }

    #[cfg(feature = "msgpack")]
    {
        use axum_serde::{MsgPack, MsgPackRaw};
        test_executor
            .execute::<MsgPack<ParametersGarde>>(Method::POST, msgpack::route::MSGPACK)
            .await?;
        test_executor
            .execute::<MsgPackRaw<ParametersGarde>>(Method::POST, msgpack::route::MSGPACK_RAW)
            .await?;
    }

    #[cfg(feature = "xml")]
    {
        use axum_serde::Xml;
        test_executor
            .execute::<Xml<ParametersGarde>>(Method::POST, xml::route::XML)
            .await?;
    }

    #[cfg(feature = "toml")]
    {
        use axum_serde::Toml;
        test_executor
            .execute::<Toml<ParametersGarde>>(Method::POST, toml::route::TOML)
            .await?;
    }

    #[cfg(feature = "sonic")]
    {
        use axum_serde::Sonic;
        test_executor
            .execute::<Sonic<ParametersGarde>>(Method::POST, sonic::route::SONIC)
            .await?;
    }

    #[cfg(feature = "cbor")]
    {
        use axum_serde::Cbor;
        test_executor
            .execute::<Cbor<ParametersGarde>>(Method::POST, cbor::route::CBOR)
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
            StatusCode::OK.as_u16(),
            "Garde '{}' test failed.",
            type_name
        );

        let error_builder = self.client.request(method.clone(), url.clone());
        let error_response = T::set_error_request(error_builder).send().await?;
        assert_eq!(
            error_response.status().as_u16(),
            T::ERROR_STATUS_CODE.as_u16(),
            "Error '{}' test failed.",
            type_name
        );

        let invalid_builder = self.client.request(method, url);
        let invalid_response = T::set_invalid_request(invalid_builder).send().await?;
        assert_eq!(
            invalid_response.status().as_u16(),
            T::INVALID_STATUS_CODE.as_u16(),
            "Invalid '{}' test failed.",
            type_name
        );
        #[cfg(feature = "into_json")]
        if T::JSON_SERIALIZABLE {
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
        response.headers()[reqwest::header::CONTENT_TYPE],
        reqwest::header::HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        "'{}' rejection into json test failed",
        type_name
    );
    assert!(response.json::<serde_json::Value>().await.is_ok());
}

mod route {
    pub const PATH: &str = "/path/:v0/:v1";
    pub const QUERY: &str = "/query";
    pub const FORM: &str = "/form";
    pub const JSON: &str = "/json";
}

async fn extract_path(Garde(Path(parameters)): Garde<Path<ParametersGarde>>) -> StatusCode {
    validate_again(parameters, ())
}

async fn extract_query(Garde(Query(parameters)): Garde<Query<ParametersGarde>>) -> StatusCode {
    validate_again(parameters, ())
}

async fn extract_form(Garde(Form(parameters)): Garde<Form<ParametersGarde>>) -> StatusCode {
    validate_again(parameters, ())
}

async fn extract_json(Garde(Json(parameters)): Garde<Json<ParametersGarde>>) -> StatusCode {
    validate_again(parameters, ())
}

fn validate_again<V: Validate>(validate: V, context: V::Context) -> StatusCode {
    // The `Garde` extractor has validated the `parameters` once,
    // it should have returned `400 BAD REQUEST` if the `parameters` were invalid,
    // Let's validate them again to check if the `Garde` extractor works well.
    // If it works properly, this function will never return `500 INTERNAL SERVER ERROR`
    match validate.validate_with(&context) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(feature = "typed_header")]
mod typed_header {
    pub(crate) mod route {
        pub const TYPED_HEADER: &str = "/typed_header";
    }

    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_extra::headers::{Error, Header, HeaderName, HeaderValue};
    use axum_extra::typed_header::TypedHeader;

    pub static AXUM_VALID_PARAMETERS: HeaderName = HeaderName::from_static("axum-valid-parameters");

    pub(super) async fn extract_typed_header(
        Garde(TypedHeader(parameters)): Garde<TypedHeader<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }

    impl Header for ParametersGarde {
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
                [v0, v1] => Ok(ParametersGarde {
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
        let parameter = ParametersGarde {
            v0: 123456,
            v1: "111111".to_string(),
        };
        let mut vec = Vec::new();
        parameter.encode(&mut vec);
        let mut iter = vec.iter();
        assert_eq!(parameter, ParametersGarde::decode(&mut iter)?);
        Ok(())
    }
}

#[cfg(feature = "typed_multipart")]
mod typed_multipart {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_typed_multipart::{BaseMultipart, TypedMultipart, TypedMultipartError};

    pub mod route {
        pub const TYPED_MULTIPART: &str = "/typed_multipart";
        pub const BASE_MULTIPART: &str = "/base_multipart";
    }

    impl From<&ParametersGarde> for reqwest::multipart::Form {
        fn from(value: &ParametersGarde) -> Self {
            reqwest::multipart::Form::new()
                .text("v0", value.v0.to_string())
                .text("v1", value.v1.clone())
        }
    }

    pub(super) async fn extract_typed_multipart(
        Garde(TypedMultipart(parameters)): Garde<TypedMultipart<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }

    pub(super) async fn extract_base_multipart(
        Garde(BaseMultipart { data, .. }): Garde<
            BaseMultipart<ParametersGarde, TypedMultipartError>,
        >,
    ) -> StatusCode {
        validate_again(data, ())
    }
}

#[cfg(feature = "extra")]
mod extra {
    use super::{validate_again, ParametersGarde};
    use crate::tests::{Rejection, ValidTest, ValidTestParameter};
    use crate::{Garde, GardeRejection};
    use axum::extract::FromRequestParts;
    use axum::http::request::Parts;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use axum_extra::extract::{Cached, WithRejection};
    use reqwest::RequestBuilder;

    pub mod route {
        pub const CACHED: &str = "/cached";
        pub const WITH_REJECTION: &str = "/with_rejection";
        pub const WITH_REJECTION_GARDE: &str = "/with_rejection_garde";
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
    impl<S> FromRequestParts<S> for ParametersGarde
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

    impl ValidTest for ParametersGarde {
        const ERROR_STATUS_CODE: StatusCode = CACHED_REJECTION_STATUS;

        fn set_valid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersGarde::valid())
                    .expect("Failed to serialize parameters"),
            )
        }

        fn set_error_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersGarde::error())
                    .expect("Failed to serialize parameters"),
            )
        }

        fn set_invalid_request(builder: RequestBuilder) -> RequestBuilder {
            builder.header(
                PARAMETERS_HEADER,
                serde_json::to_string(ParametersGarde::invalid())
                    .expect("Failed to serialize parameters"),
            )
        }
    }

    pub struct GardeWithRejectionRejection {
        inner: ParametersRejection,
    }

    impl Rejection for GardeWithRejectionRejection {
        const STATUS_CODE: StatusCode = StatusCode::CONFLICT;
    }

    impl IntoResponse for GardeWithRejectionRejection {
        fn into_response(self) -> Response {
            let mut response = self.inner.into_response();
            *response.status_mut() = Self::STATUS_CODE;
            response
        }
    }

    // satisfy the `WithRejection`'s extractor trait bound
    // R: From<E::Rejection> + IntoResponse
    impl From<ParametersRejection> for GardeWithRejectionRejection {
        fn from(inner: ParametersRejection) -> Self {
            Self { inner }
        }
    }

    pub async fn extract_cached(
        Garde(Cached(parameters)): Garde<Cached<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }

    pub async fn extract_with_rejection(
        Garde(WithRejection(parameters, _)): Garde<
            WithRejection<ParametersGarde, GardeWithRejectionRejection>,
        >,
    ) -> StatusCode {
        validate_again(parameters, ())
    }

    pub struct WithRejectionGardeRejection<E> {
        inner: GardeRejection<E>,
    }

    impl<E> From<GardeRejection<E>> for WithRejectionGardeRejection<E> {
        fn from(inner: GardeRejection<E>) -> Self {
            Self { inner }
        }
    }

    impl<E: IntoResponse> IntoResponse for WithRejectionGardeRejection<E> {
        fn into_response(self) -> Response {
            let mut res = self.inner.into_response();
            *res.status_mut() = StatusCode::IM_A_TEAPOT;
            res
        }
    }

    pub async fn extract_with_rejection_valid(
        WithRejection(Garde(parameters), _): WithRejection<
            Garde<ParametersGarde>,
            WithRejectionGardeRejection<ParametersRejection>,
        >,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "extra_typed_path")]
mod extra_typed_path {
    use super::validate_again;
    use crate::{Garde, HasValidate};
    use axum::http::StatusCode;
    use axum_extra::routing::TypedPath;
    use garde::Validate;
    use serde::Deserialize;

    pub mod route {
        pub const EXTRA_TYPED_PATH: &str = "/extra_typed_path/:v0/:v1";
    }

    #[derive(Validate, TypedPath, Deserialize)]
    #[typed_path("/extra_typed_path/:v0/:v1")]
    pub struct TypedPathParam {
        #[garde(range(min = 5, max = 10))]
        v0: i32,
        #[garde(length(min = 1, max = 10))]
        v1: String,
    }

    impl HasValidate for TypedPathParam {
        type Validate = Self;

        fn get_validate(&self) -> &Self::Validate {
            self
        }
    }

    pub async fn extract_extra_typed_path(Garde(param): Garde<TypedPathParam>) -> StatusCode {
        validate_again(param, ())
    }
}

#[cfg(feature = "extra_query")]
mod extra_query {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_extra::extract::Query;

    pub mod route {
        pub const EXTRA_QUERY: &str = "/extra_query";
    }

    pub async fn extract_extra_query(
        Garde(Query(parameters)): Garde<Query<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "extra_form")]
mod extra_form {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_extra::extract::Form;

    pub mod route {
        pub const EXTRA_FORM: &str = "/extra_form";
    }

    pub async fn extract_extra_form(
        Garde(Form(parameters)): Garde<Form<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "extra_protobuf")]
mod extra_protobuf {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_extra::protobuf::Protobuf;

    pub mod route {
        pub const EXTRA_PROTOBUF: &str = "/extra_protobuf";
    }

    pub async fn extract_extra_protobuf(
        Garde(Protobuf(parameters)): Garde<Protobuf<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "yaml")]
mod yaml {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::Yaml;

    pub mod route {
        pub const YAML: &str = "/yaml";
    }

    pub async fn extract_yaml(Garde(Yaml(parameters)): Garde<Yaml<ParametersGarde>>) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "msgpack")]
mod msgpack {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::{MsgPack, MsgPackRaw};

    pub mod route {
        pub const MSGPACK: &str = "/msgpack";
        pub const MSGPACK_RAW: &str = "/msgpack_raw";
    }

    pub async fn extract_msgpack(
        Garde(MsgPack(parameters)): Garde<MsgPack<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }

    pub async fn extract_msgpack_raw(
        Garde(MsgPackRaw(parameters)): Garde<MsgPackRaw<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "xml")]
mod xml {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::Xml;

    pub mod route {
        pub const XML: &str = "/xml";
    }

    pub async fn extract_xml(Garde(Xml(parameters)): Garde<Xml<ParametersGarde>>) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "toml")]
mod toml {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::Toml;

    pub mod route {
        pub const TOML: &str = "/toml";
    }

    pub async fn extract_toml(Garde(Toml(parameters)): Garde<Toml<ParametersGarde>>) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "sonic")]
mod sonic {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::Sonic;

    pub mod route {
        pub const SONIC: &str = "/sonic";
    }

    pub async fn extract_sonic(
        Garde(Sonic(parameters)): Garde<Sonic<ParametersGarde>>,
    ) -> StatusCode {
        validate_again(parameters, ())
    }
}

#[cfg(feature = "cbor")]
mod cbor {
    use super::{validate_again, ParametersGarde};
    use crate::Garde;
    use axum::http::StatusCode;
    use axum_serde::Cbor;

    pub mod route {
        pub const CBOR: &str = "/cbor";
    }

    pub async fn extract_cbor(Garde(Cbor(parameters)): Garde<Cbor<ParametersGarde>>) -> StatusCode {
        validate_again(parameters, ())
    }
}
