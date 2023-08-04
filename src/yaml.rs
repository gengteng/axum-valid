//! # Implementation of the `HasValidate` trait for the `Yaml` extractor.
//!

use crate::HasValidate;
use axum_yaml::Yaml;
use validator::Validate;

impl<T: Validate> HasValidate for Yaml<T> {
    type Validate = T;
    fn get_validate(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{ValidTest, ValidTestParameter};
    use axum::http::StatusCode;
    use axum_yaml::Yaml;
    use reqwest::RequestBuilder;

    impl<T: ValidTestParameter> ValidTest for Yaml<T> {
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
