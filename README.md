# axum-valid

The Valid crate provides a `Valid` type that can be used in combination with `Json`, `Path`, `Query`, and `Form` types to validate the entities that implement the `Validate` trait.

## Usage

```rust
use validator::Validate;
use serde::Deserialize;

#[derive(Debug, Validate, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pager {
    #[validate(range(min = 1, max = 50))]
    page_size: usize,
    #[validate(range(min = 1))]
    page_no: usize,
}

pub async fn get_page_by_query(
    Valid(Query(pager)): Valid<Query<Pager>>,
) {
    assert!((1..=50).contains(pager.page_size));
    assert!((1..).contains(pager.page_no));
}

pub async fn get_page_by_json(
    Valid(Json(pager)): Valid<Json<Pager>>,
) {
    assert!((1..=50).contains(pager.page_size));
    assert!((1..).contains(pager.page_no));
}
```