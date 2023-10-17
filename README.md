# axum-valid

[![crates.io](https://img.shields.io/crates/v/axum-valid)](https://crates.io/crates/axum-valid)
[![crates.io download](https://img.shields.io/crates/d/axum-valid)](https://crates.io/crates/axum-valid)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue)](https://github.com/gengteng/axum-valid/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/gengteng/axum-valid/status.svg)](https://deps.rs/repo/github/gengteng/axum-valid)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gengteng/axum-valid/.github/workflows/main.yml?branch=main)](https://github.com/gengteng/axum-valid/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/gengteng/axum-valid/badge.svg?branch=main)](https://coveralls.io/github/gengteng/axum-valid?branch=main)

This crate provides data validation capabilities for Axum based on the `validator` and `garde` crates. It offers the `Valid`, `ValidEx` and `Garde` types to enable validation for extractors like `Json`, `Path`, `Query` and `Form`.

`validator` support is included by default. To use `garde`, enable it via the `garde` feature. `garde` alone can be enabled with `default-features = false`.

The `Valid` type performs validation using `validator`. The `ValidEx` type supports validations requiring extra arguments. The `Garde` type unifies both argument and non-argument validations using `garde`.

Additional extractors like `TypedHeader`, `MsgPack` and `Yaml` are also supported through optional features. Refer to `Features` for details.

## Basic usage

```shell
cargo add axum-valid
```

```rust,no_run
#[cfg(feature = "validator")]
mod validator_example {
    use validator::Validate;
    use serde::Deserialize;
    use axum_valid::Valid;
    use axum::extract::Query;
    use axum::{Json, Router};
    use axum::routing::{get, post};
    
    #[derive(Debug, Validate, Deserialize)]
    pub struct Pager {
        #[validate(range(min = 1, max = 50))]
        pub page_size: usize,
        #[validate(range(min = 1))]
        pub page_no: usize,
    }
    
    pub async fn pager_from_query(
        Valid(Query(pager)): Valid<Query<Pager>>,
    ) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
    }
    
    pub async fn pager_from_json(
        pager: Valid<Json<Pager>>,
    ) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
        // NOTE: support automatic dereferencing
        println!("page_no: {}, page_size: {}", pager.page_no, pager.page_size);
    }
    
    pub fn router() -> Router {
        Router::new()
            .route("/query", get(pager_from_query))
            .route("/json", post(pager_from_json))
    }
}

#[cfg(feature = "garde")]
mod garde_example {
    use garde::Validate;
    use serde::Deserialize;
    use axum_valid::Garde;
    use axum::extract::Query;
    use axum::{Json, Router};
    use axum::routing::{get, post};
    
    #[derive(Debug, Validate, Deserialize)]
    pub struct Pager {
        #[garde(range(min = 1, max = 50))]
        pub page_size: usize,
        #[garde(range(min = 1))]
        pub page_no: usize,
    }
    
    pub async fn pager_from_query(
        Garde(Query(pager)): Garde<Query<Pager>>,
    ) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
    }
    
    pub async fn pager_from_json(
        pager: Garde<Json<Pager>>,
    ) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
        // NOTE: support automatic dereferencing
        println!("page_no: {}, page_size: {}", pager.page_no, pager.page_size);
    }
    
    pub fn router() -> Router {
        Router::new()
            .route("/query", get(pager_from_query))
            .route("/json", post(pager_from_json))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::Router;
    
    let router = Router::new();
    #[cfg(feature = "validator")]
    let router = router.nest("/validator", validator_example::router());
    #[cfg(feature = "garde")]
    let router = router.nest("/garde", garde_example::router());
    axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
```

When validation errors occur, the extractor will automatically return 400 with validation errors as the HTTP message body.

To see how each extractor can be used with `Valid`, please refer to the example in the [documentation](https://docs.rs/axum-valid) of the corresponding module.

## Argument-Based Validation

Here are the examples of using the `ValidEx` / `Garde` extractor to validate data in a `Form` using arguments:

```rust,no_run
#[cfg(feature = "validator")]
mod validator_example {
    use axum::routing::post;
    use axum::{Form, Router};
    use axum_valid::{Arguments, ValidEx};
    use serde::Deserialize;
    use std::ops::{RangeFrom, RangeInclusive};
    use validator::{Validate, ValidateArgs, ValidationError};
    
    // NOTE: When some fields use custom validation functions with arguments,
    // `#[derive(Validate)]` will implement `ValidateArgs` instead of `Validate` for the type.
    // The validation arguments will be a tuple of all the field validation args.
    // In this example it is (&RangeInclusive<usize>, &RangeFrom<usize>).
    // For more detailed information and understanding of `ValidateArgs` and their argument types, 
    // please refer to the `validator` crate documentation.
    #[derive(Debug, Validate, Deserialize)]
    pub struct Pager {
        #[validate(custom(function = "validate_page_size", arg = "&'v_a RangeInclusive<usize>"))]
        pub page_size: usize,
        #[validate(custom(function = "validate_page_no", arg = "&'v_a RangeFrom<usize>"))]
        pub page_no: usize,
    }
    
    fn validate_page_size(v: usize, args: &RangeInclusive<usize>) -> Result<(), ValidationError> {
        args.contains(&v)
            .then_some(())
            .ok_or_else(|| ValidationError::new("page_size is out of range"))
    }
    
    fn validate_page_no(v: usize, args: &RangeFrom<usize>) -> Result<(), ValidationError> {
        args.contains(&v)
            .then_some(())
            .ok_or_else(|| ValidationError::new("page_no is out of range"))
    }
    
    // NOTE: Clone is required
    #[derive(Debug, Clone)]
    pub struct PagerValidArgs {
        page_size_range: RangeInclusive<usize>,
        page_no_range: RangeFrom<usize>,
    }
    
    // NOTE: This implementation allows PagerValidArgs to be the second member of ValidEx, and provides arguments for actual validation.
    // The type mapping <Pager as ValidateArgs<'a>>::Args represents the combination of validators applied on each field of Pager.
    // get() method returns the validating arguments to be used during validation.
    impl<'a> Arguments<'a> for PagerValidArgs {
        type T = Pager;
    
        // NOTE: <Pager as ValidateArgs<'a>>::Args == (&RangeInclusive<usize>, &RangeFrom<usize>)
        fn get(&'a self) -> <Pager as ValidateArgs<'a>>::Args {
            (&self.page_size_range, &self.page_no_range)
        }
    }
    
    pub async fn pager_from_form_ex(ValidEx(Form(pager), _): ValidEx<Form<Pager>, PagerValidArgs>) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
    }
    
    pub fn router() -> Router {
        Router::new()
            .route("/form", post(pager_from_form_ex))
            .with_state(PagerValidArgs {
                page_size_range: 1..=50,
                page_no_range: 1..,
            })
        // NOTE: The PagerValidArgs can also be stored in a XxxState,
        // make sure it implements FromRef<XxxState>.
        // Consider using Arc to reduce deep copying costs.
    }
}

#[cfg(feature = "garde")]
mod garde_example {
    use axum::routing::post;
    use axum::{Form, Router};
    use axum_valid::Garde;
    use garde::Validate;
    use serde::Deserialize;
    use std::ops::{RangeFrom, RangeInclusive};

    #[derive(Debug, Validate, Deserialize)]
    #[garde(context(PagerValidContext))]
    pub struct Pager {
        #[garde(custom(validate_page_size))]
        pub page_size: usize,
        #[garde(custom(validate_page_no))]
        pub page_no: usize,
    }

    fn validate_page_size(v: &usize, args: &PagerValidContext) -> garde::Result {
        args.page_size_range
            .contains(&v)
            .then_some(())
            .ok_or_else(|| garde::Error::new("page_size is out of range"))
    }

    fn validate_page_no(v: &usize, args: &PagerValidContext) -> garde::Result {
        args.page_no_range
            .contains(&v)
            .then_some(())
            .ok_or_else(|| garde::Error::new("page_no is out of range"))
    }

    #[derive(Debug, Clone)]
    pub struct PagerValidContext {
        page_size_range: RangeInclusive<usize>,
        page_no_range: RangeFrom<usize>,
    }

    pub async fn pager_from_form_garde(Garde(Form(pager)): Garde<Form<Pager>>) {
        assert!((1..=50).contains(&pager.page_size));
        assert!((1..).contains(&pager.page_no));
    }

    pub fn router() -> Router {
        Router::new()
            .route("/form", post(pager_from_form_garde))
            .with_state(PagerValidContext {
                page_size_range: 1..=50,
                page_no_range: 1..,
            })
        // NOTE: The PagerValidContext can also be stored in a XxxState,
        // make sure it implements FromRef<XxxState>.
        // Consider using Arc to reduce deep copying costs.
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::Router;
    let router = Router::new();
    #[cfg(feature = "validator")]
    let router = router.nest("/validator", validator_example::router());
    #[cfg(feature = "garde")]
    let router = router.nest("/garde", garde_example::router());
    axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
```

Current module documentation predominantly showcases `Valid` examples, the usage of `ValidEx` is analogous.

## Features

| Feature          | Description                                                                                                                              | Module                                       | Default | Example | Tests |
|------------------|------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------|---------|---------|-------|
| default          | Enables `validator` and support for `Query`, `Json` and `Form`                                                                           | [`validator`], [`query`], [`json`], [`form`] | ✅       | ✅       | ✅     |
| validator        | Enables `validator` (`Valid`, `ValidEx`)                                                                                                 | [`validator`]                                | ✅       | ✅       | ✅     |
| garde            | Enables `garde` (`Garde`)                                                                                                                | [`garde`]                                    | ❌       | ✅       | ✅     |
| validify         | Enables `validify` (`Validated`, `Modified`, `Validified`)                                                                               | [`validify`]                                 | ❌       | ✅       | ✅     |
| basic            | Enables support for `Query`, `Json` and `Form`                                                                                           | [`query`], [`json`], [`form`]                | ✅       | ✅       | ✅     |
| json             | Enables support for `Json`                                                                                                               | [`json`]                                     | ✅       | ✅       | ✅     |
| query            | Enables support for `Query`                                                                                                              | [`query`]                                    | ✅       | ✅       | ✅     |
| form             | Enables support for `Form`                                                                                                               | [`form`]                                     | ✅       | ✅       | ✅     |
| typed_header     | Enables support for `TypedHeader`                                                                                                        | [`typed_header`]                             | ❌       | ✅       | ✅     |
| typed_multipart  | Enables support for `TypedMultipart` and `BaseMultipart` from `axum_typed_multipart`                                                     | [`typed_multipart`]                          | ❌       | ✅       | ✅     |
| msgpack          | Enables support for `MsgPack` and `MsgPackRaw` from `axum-msgpack`                                                                       | [`msgpack`]                                  | ❌       | ✅       | ✅     |
| yaml             | Enables support for `Yaml` from `axum-yaml`                                                                                              | [`yaml`]                                     | ❌       | ✅       | ✅     |
| extra            | Enables support for `Cached`, `WithRejection` from `axum-extra`                                                                          | [`extra`]                                    | ❌       | ✅       | ✅     |
| extra_typed_path | Enables support for `T: TypedPath` from `axum-extra`                                                                                     | [`extra::typed_path`]                        | ❌       | ✅       | ✅     |
| extra_query      | Enables support for `Query` from `axum-extra`                                                                                            | [`extra::query`]                             | ❌       | ✅       | ✅     |
| extra_form       | Enables support for `Form` from `axum-extra`                                                                                             | [`extra::form`]                              | ❌       | ✅       | ✅     |
| extra_protobuf   | Enables support for `Protobuf` from `axum-extra`                                                                                         | [`extra::protobuf`]                          | ❌       | ✅       | ✅     |
| all_extra_types  | Enables support for all extractors above from `axum-extra`                                                                               | N/A                                          | ❌       | ✅       | ✅     |
| all_types        | Enables support for all extractors above                                                                                                 | N/A                                          | ❌       | ✅       | ✅     |
| 422              | Use `422 Unprocessable Entity` instead of `400 Bad Request` as the status code when validation fails                                     | [`VALIDATION_ERROR_STATUS`]                  | ❌       | ✅       | ✅     |
| into_json        | Validation errors will be serialized into JSON format and returned as the HTTP body                                                      | N/A                                          | ❌       | ✅       | ✅     |
| full             | Enables `all_types`, `422` and `into_json`                                                                                               | N/A                                          | ❌       | ✅       | ✅     |
| full_garde       | Enables `garde`, `all_types`, `422` and `into_json`. Consider using `default-features = false` to exclude default `validator` support    | N/A                                          | ❌       | ✅       | ✅     |
| full_garde       | Enables `validify`, `all_types`, `422` and `into_json`. Consider using `default-features = false` to exclude default `validator` support | N/A                                          | ❌       | ✅       | ✅     |

## Compatibility

To determine the compatible versions of `axum-valid`, `axum-extra`, `axum-yaml` and other dependencies that work together, please refer to the dependencies listed in the `Cargo.toml` file. The version numbers listed there will indicate the compatible versions.

## License

This project is licensed under the MIT License.

## References

* [axum](https://crates.io/crates/axum)
* [validator](https://crates.io/crates/validator)
* [garde](https://crates.io/crates/garde)
* [serde](https://crates.io/crates/serde)
* [axum-extra](https://crates.io/crates/axum-extra)
* [axum-yaml](https://crates.io/crates/axum-yaml)
* [axum-msgpack](https://crates.io/crates/axum-msgpack)
* [axum_typed_multipart](https://crates.io/crates/axum_typed_multipart)
