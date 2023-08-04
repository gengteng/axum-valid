# axum-valid

[![crates.io](https://img.shields.io/crates/v/axum-valid)](https://crates.io/crates/axum-valid)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue)](https://github.com/gengteng/axum-valid/blob/main/LICENSE)
[![dependency status](https://deps.rs/repo/github/gengteng/axum-valid/status.svg)](https://deps.rs/repo/github/gengteng/axum-valid)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gengteng/axum-valid/.github/workflows/main.yml?branch=main)](https://github.com/gengteng/axum-valid/actions/workflows/ci.yml)
[![Coverage Status](https://coveralls.io/repos/github/gengteng/axum-valid/badge.svg?branch=main)](https://coveralls.io/github/gengteng/axum-valid?branch=main)

This crate provides a `Valid` type that can be used in combination with `Json`, `Path`, `Query`, and `Form` extractors to validate the entities that implement the `Validate` trait from the `validator` crate.

Additional extractors like `TypedHeader`, `MsgPack`, `Yaml` etc. are supported through optional features. The full list of supported extractors is in the Features section below.

## Usage

```shell
cargo add axum-valid
```

```rust
use validator::Validate;
use serde::Deserialize;
use axum_valid::Valid;
use axum::extract::Query;
use axum::Json;

#[derive(Debug, Validate, Deserialize)]
pub struct Pager {
    #[validate(range(min = 1, max = 50))]
    pub page_size: usize,
    #[validate(range(min = 1))]
    pub page_no: usize,
}

pub async fn get_page_by_query(
    Valid(Query(pager)): Valid<Query<Pager>>,
) {
    assert!((1..=50).contains(&pager.page_size));
    assert!((1..).contains(&pager.page_no));
}

pub async fn get_page_by_json(
    Valid(Json(pager)): Valid<Json<Pager>>,
) {
    assert!((1..=50).contains(&pager.page_size));
    assert!((1..).contains(&pager.page_no));
}
```

When validation errors occur, the extractor will automatically return 400 with validation errors as the HTTP message body.

## Features

| Feature         | Description                                                                                          | Default | Tests |
|-----------------|------------------------------------------------------------------------------------------------------|---------|-------|
| default         | Enables support for `Path`, `Query`, `Json` and `Form`                                               | ✅       | ✅     |
| json            | Enables support for `Json`                                                                           | ✅       | ✅     |
| query           | Enables support for `Query`                                                                          | ✅       | ✅     |
| form            | Enables support for `Form`                                                                           | ✅       | ✅     |
| typed_header    | Enables support for `TypedHeader`                                                                    | ❌       | ✅     |
| msgpack         | Enables support for `MsgPack` and `MsgPackRaw` from `axum-msgpack`                                   | ❌       | ✅     |
| yaml            | Enables support for `Yaml` from `axum-yaml`                                                          | ❌       | ✅     |
| extra           | Enables support for `Cached`, `WithRejection` from `axum-extra`                                      | ❌       | ✅     |
| extra_query     | Enables support for `Query` from `axum-extra`                                                        | ❌       | ✅     |
| extra_form      | Enables support for `Form` from `axum-extra`                                                         | ❌       | ✅     |
| extra_protobuf  | Enables support for `Protobuf` from `axum-extra`                                                     | ❌       | ✅     |
| all_extra_types | Enables support for all extractors above from `axum-extra`                                           | ❌       | ✅     |
| all_types       | Enables support for all extractors above                                                             | ❌       | ✅     |
| 422             | Use `422 Unprocessable Entity` instead of `400 Bad Request` as the status code when validation fails | ❌       | ✅     |
| into_json       | Validation errors will be serialized into JSON format and returned as the HTTP body                  | ❌       | ✅     |
| full            | Enables all features                                                                                 | ❌       | ✅     |

## License

This project is licensed under the MIT License.

## References

* [axum](https://crates.io/crates/axum)
* [validator](https://crates.io/crates/validator)
* [serde](https://crates.io/crates/serde)