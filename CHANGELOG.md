# Changelog

All notable changes to this project will be documented in this file.\
This project adheres to [Semantic Versioning](https://semver.org/).

---

[1.1.0]: https://github.com/qbasic16/swiss_uid/releases/tag/1.1.0
[1.0.2]: https://github.com/qbasic16/swiss_uid/releases/tag/1.0.2
[1.0.1]: https://github.com/qbasic16/swiss_uid/releases/tag/1.0.1
[1.0.0]: https://github.com/qbasic16/swiss_uid/releases/tag/1.0.0

## [1.1.0] - 2025-01-26

### Breaking

- Changed method signature and body of `SwissUid::checkdigit` to return the
  contained checkdigit instead of recalculating it

### Changed

- Refactored nibble utils
- Cleaned up code

## [1.0.2] - 2024-07-07

### Added

- Added missing check for leading `0` (not allowed by the standard)

### Fixed

- Fixed missing zeroes in the `Debug` and `Display` representation

## [1.0.1] - 2024-07-07

### Added

- Added feature flags `default` and `rand`
- Added integration tests

### Changed

- Refactored code and parsing
- Changed struct layout and decreased struct memory footprint massively

## [1.0.0] - 2024-01-22

Initial release containing a simple implementation of `SwissUid`
