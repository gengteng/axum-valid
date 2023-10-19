# Changelog

## Unreleased

### Added

### Changed

### Fixed

## axum-valid 0.11.0 (2023-10-20)

### Added

* Added support for `validify`.
* Introduced four new extractors using validify: `Validated`, `Modified`, `Validified`, and `ValidifiedByRef`.

### Changed

### Fixed

## axum-valid 0.10.1 (2023-10-10)

### Added

### Changed

* When enabling features starting with `extra_` like `extra_query`, the `extra` feature will now be automatically enabled. Previously, users had to manually enable both `extra` and `extra_*`.

### Fixed

## axum-valid 0.10.0 (2023-10-09)

### Added

* Added support for `garde`.

### Changed

* Refactored the module structure.

### Fixed

## axum-valid 0.9.0 (2023-09-29)

### Added

* Introduced the `ValidEx` type to enhance data validation capabilities.
* Added the `Arguments` and `HasValidateArgs` traits to support the use of `ValidEx`.

### Changed

* Upgraded `axum_typed_multipart` to version 0.10.0.

### Fixed

## axum-valid 0.8.0 (2023-09-19)

### Added

* Upgraded `axum-extra` dependencies to version 0.8.0

### Changed

### Fixed

## axum-valid 0.7.0 (2023-09-12)

### Added

* Support for `TypedPath` from `axum-extra`.
* Documentation for each module, including usage and examples.

### Changed

### Fixed

## axum-valid 0.6.0 (2023-09-04)

### Added

* Support for `TypedMultipart` and `BaseMultipart` from `axum_typed_multipart`

### Changed

### Fixed

## axum-valid 0.5.1 (2023-08-05)

### Added

* Support for more extractors
  * `TypedHeader`
  * `MsgPack` and `MsgPackRaw` from `axum-msgpack`
  * `Yaml` from `axum-yaml`
  * `Cached`, `WithRejection`, `Query`, `Form` and `Protobuf` from `axum-extra`
* Tests and feature flags

### Changed

### Fixed

## axum-valid 0.4.2 (2023-08-01)

### Added

* Feature flags
  * `422`: Use 422 Unprocessable Entity instead of 400 Bad Request as the status code when validation fails.
  * `into_json`: When this feature is enabled, validation errors will be serialized into JSON format and returned as the HTTP body.

### Changed

### Fixed