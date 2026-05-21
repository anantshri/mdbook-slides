# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-21

### Changed
- **Support mdbook 0.5 series.** `mdbook` 0.5 no longer ships a library target; the preprocessor now depends on `mdbook-preprocessor 0.5` instead. The `Preprocessor::supports_renderer` trait method also changed signature from `-> bool` to `-> Result<bool>`, and `CmdPreprocessor::parse_input` is replaced by the free function `mdbook_preprocessor::parse_input`.
- Bumped `pulldown-cmark` 0.11 → 0.13.
- Bumped `toml_edit` 0.22 → 0.23.
- Replaced deprecated `serde_yaml` with `serde_yaml_ng 0.10` (drop-in maintained fork).

### Compatibility
- **Breaking:** requires mdbook 0.5.0 or newer. Users still on mdbook 0.4.x should pin `mdbook-slides = "0.1"`.

## [0.1.1] - 2026-03-16

Initial public release.

[0.2.0]: https://github.com/anantshri/mdbook-slides/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/anantshri/mdbook-slides/releases/tag/v0.1.1
