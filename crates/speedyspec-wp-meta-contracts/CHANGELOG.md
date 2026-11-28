# Changelog

## [0.4.0](https://github.com/SpeedySpec/speedyspec-platform/compare/speedyspec-wp-meta-contracts-v0.3.0...speedyspec-wp-meta-contracts-v0.4.0) (2026-11-28)


### ⚠ BREAKING CHANGES

* **meta-contracts:** MetaKeys requires subtypes(); MetaRepository::for_objects returns Vec<MetaRow> instead of Vec<Meta>; MetaRepository requires ids_of_object().

### Features

* **meta-contracts:** subtypes on MetaKeys, WordPress's cache-priming read, and an object's meta ids ([#25](https://github.com/SpeedySpec/speedyspec-platform/issues/25)) ([37d5a3d](https://github.com/SpeedySpec/speedyspec-platform/commit/37d5a3d7ebadac4e87214582f5a4b8028bb54313))

## [0.3.0](https://github.com/SpeedySpec/speedyspec-platform/compare/speedyspec-wp-meta-contracts-v0.2.0...speedyspec-wp-meta-contracts-v0.3.0) (2026-11-26)


### Features

* **meta-contracts:** the Metadata surface and the registered-keys read port ([#12](https://github.com/SpeedySpec/speedyspec-platform/issues/12)) ([978df4e](https://github.com/SpeedySpec/speedyspec-platform/commit/978df4e2e35e03cabc7fbce4acabda99cfe2abb7))

## [0.2.0](https://github.com/SpeedySpec/speedyspec-platform/compare/speedyspec-wp-meta-contracts-v0.1.0...speedyspec-wp-meta-contracts-v0.2.0) (2026-11-26)


### Miscellaneous Chores

* publish readiness for every crate ([#6](https://github.com/SpeedySpec/speedyspec-platform/issues/6)) ([b93a8a0](https://github.com/SpeedySpec/speedyspec-platform/commit/b93a8a0caac9eb0d534d1892cffed0d3d2321241))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * speedyspec-php-serialize bumped from 0.2.0 to 0.3.0
