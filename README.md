# axum-valid

This crate provides a `Valid` type that can be used in combination with `Json`, `Path`, `Query`, and `Form` types to validate the entities that implement the `Validate` trait.

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

For more usage examples, please refer to the `basic.rs` and `custom.rs` files in the `tests` directory.