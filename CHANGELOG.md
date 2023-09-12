# Changelog

## Unreleased

### Added

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