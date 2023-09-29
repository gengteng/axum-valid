# axum-valid

[![crates.io](https://img.shields.io/crates/v/axum-valid)](https://crates.io/crates/axum-valid)
[![crates.io download](https://img.shields.io/crates/d/axum-valid)](https://crates.io/crates/axum-valid)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue)](https://github.com/gengteng/axum-valid/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/gengteng/axum-valid/status.svg)](https://deps.rs/repo/github/gengteng/axum-valid)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gengteng/axum-valid/.github/workflows/main.yml?branch=main)](https://github.com/gengteng/axum-valid/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/gengteng/axum-valid/badge.svg?branch=main)](https://coveralls.io/github/gengteng/axum-valid?branch=main)

This crate provides a `Valid` type that can be used in combination with `Json`, `Path`, `Query`, and `Form` extractors to validate the entities that implement the `Validate` trait from the `validator` crate.

It also provides a `ValidEx` type that works similarly to `Valid`, but can perform validation requiring additional arguments by using types that implement the `ValidateArgs` trait.

Additional extractors like `TypedHeader`, `MsgPack`, `Yaml` etc. are supported through optional features. The full list of supported extractors is in the Features section below.

## Basic usage

```shell
cargo add axum-valid
```

```rust,no_run
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
    Valid(Json(pager)): Valid<Json<Pager>>,
) {
    assert!((1..=50).contains(&pager.page_size));
    assert!((1..).contains(&pager.page_no));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .route("/query", get(pager_from_query))
        .route("/json", post(pager_from_json));
    axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
```

When validation errors occur, the extractor will automatically return 400 with validation errors as the HTTP message body.

To see how each extractor can be used with `Valid`, please refer to the example in the [documentation](https://docs.rs/axum-valid) of the corresponding module.

## Argument-Based Validation

Here's a basic example of using the `ValidEx` extractor to validate data in a `Form` using arguments:

```rust,no_run
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new()
        .route("/form", post(pager_from_form_ex))
        .with_state(PagerValidArgs {
            page_size_range: 1..=50,
            page_no_range: 1..,
        });
    // NOTE: The PagerValidArgs can also be stored in a XxxState,
    // make sure it implements FromRef<XxxState>.

    axum::Server::bind(&([0u8, 0, 0, 0], 8080).into())
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
```

## Features

| Feature          | Description                                                                                          | Default | Example | Tests |
|------------------|------------------------------------------------------------------------------------------------------|---------|---------|-------|
| default          | Enables support for `Path`, `Query`, `Json` and `Form`                                               | ✅       | ✅       | ✅     |
| json             | Enables support for `Json`                                                                           | ✅       | ✅       | ✅     |
| query            | Enables support for `Query`                                                                          | ✅       | ✅       | ✅     |
| form             | Enables support for `Form`                                                                           | ✅       | ✅       | ✅     |
| typed_header     | Enables support for `TypedHeader`                                                                    | ❌       | ✅       | ✅     |
| typed_multipart  | Enables support for `TypedMultipart` and `BaseMultipart` from `axum_typed_multipart`                 | ❌       | ✅       | ✅     |
| msgpack          | Enables support for `MsgPack` and `MsgPackRaw` from `axum-msgpack`                                   | ❌       | ✅       | ✅     |
| yaml             | Enables support for `Yaml` from `axum-yaml`                                                          | ❌       | ✅       | ✅     |
| extra            | Enables support for `Cached`, `WithRejection` from `axum-extra`                                      | ❌       | ✅       | ✅     |
| extra_typed_path | Enables support for `T: TypedPath` from `axum-extra`                                                 | ❌       | ✅       | ✅     |
| extra_query      | Enables support for `Query` from `axum-extra`                                                        | ❌       | ✅       | ✅     |
| extra_form       | Enables support for `Form` from `axum-extra`                                                         | ❌       | ✅       | ✅     |
| extra_protobuf   | Enables support for `Protobuf` from `axum-extra`                                                     | ❌       | ✅       | ✅     |
| all_extra_types  | Enables support for all extractors above from `axum-extra`                                           | ❌       | ✅       | ✅     |
| all_types        | Enables support for all extractors above                                                             | ❌       | ✅       | ✅     |
| 422              | Use `422 Unprocessable Entity` instead of `400 Bad Request` as the status code when validation fails | ❌       | ✅       | ✅     |
| into_json        | Validation errors will be serialized into JSON format and returned as the HTTP body                  | ❌       | ✅       | ✅     |
| full             | Enables all features                                                                                 | ❌       | ✅       | ✅     |

## Compatibility

To determine the compatible versions of `axum-valid`, `axum-extra`, `axum-yaml` and other dependencies that work together, please refer to the dependencies listed in the `Cargo.toml` file. The version numbers listed there will indicate the compatible versions.

## License

This project is licensed under the MIT License.

## References

* [axum](https://crates.io/crates/axum)
* [validator](https://crates.io/crates/validator)
* [serde](https://crates.io/crates/serde)
* [axum-extra](https://crates.io/crates/axum-extra)
* [axum-yaml](https://crates.io/crates/axum-yaml)
* [axum-msgpack](https://crates.io/crates/axum-msgpack)
* [axum_typed_multipart](https://crates.io/crates/axum_typed_multipart)
