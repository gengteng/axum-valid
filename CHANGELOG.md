# Changelog

## Unreleased

### Added

### Changed

### Fixed

## axum-valid 0.18.0 (2024-04-14)

### Added

### Changed

* Upgrade validator to 0.18.1.
* Upgrade validify to 1.4.0.
* Upgrade axum-serde to 0.4.1.
    * Add support for `Cbor<T>`.

## axum-valid 0.17.0 (2024-03-05)

### Added

### Changed

* Upgrade validator to 0.17.0.
* Refactor argument-based validation using validator.

### Fixed

## axum-valid 0.16.0 (2024-03-01)

### Added

* Add support for `Sonic<T>` from axum-serde.

### Changed

* Upgrade axum-serde to 0.3.0.
* Remove unnecessary development dependencies.

### Fixed

## axum-valid 0.15.1 (2024-02-28)

### Added

### Changed

### Fixed

* Fix the compilation error on docs.rs.

## axum-valid 0.15.0 (2024-02-01)

### Added

### Changed

* Upgrade garde to 0.18.0.

### Fixed

## axum-valid 0.14.0 (2024-01-06)

### Added

### Changed

* Add support for aide through the 'aide' feature.
* Upgrade garde to 0.17.0.
* Upgrade validify to 1.3.0. (This update also resulted in some refactoring. Please refer to the `README.md` for the
  latest examples.)

### Fixed

## axum-valid 0.13.0 (2023-12-14)

### Added

### Changed

* Update axum-serde from 0.1.0 to 0.2.0.

### Fixed

## axum-valid 0.12.0 (2023-12-04)

### Added

* Add Support for Xml, Toml from axum-serde.

### Changed

* Update axum from 0.6 to 0.7.
* Use axum-serde instead of axum-yaml and axum-msgpack.

### Fixed

## axum-valid 0.11.0 (2023-10-21)

### Added

* Added support for `validify`.
* Introduced four new extractors using validify: `Validated`, `Modified`, `Validified`, and `ValidifiedByRef`.

### Changed

### Fixed

## axum-valid 0.10.1 (2023-10-10)

### Added

### Changed

* When enabling features starting with `extra_` like `extra_query`, the `extra` feature will now be automatically
  enabled. Previously, users had to manually enable both `extra` and `extra_*`.

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
    * `into_json`: When this feature is enabled, validation errors will be serialized into JSON format and returned as
      the HTTP body.

### Changed

### Fixed