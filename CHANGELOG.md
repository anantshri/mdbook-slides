# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] - 2026-09-01

### Added
- **On-slide navigation cues and orientation.** Slides now show full-height
  left/right click zones with arrow glyphs (`‹` / `›`) — click a side (or use the
  arrow keys / space, as before) to move between slides, including across chapter
  boundaries. A bottom-left pill shows the current chapter name so you stay
  oriented even with the sidebar collapsed. All hidden in print/PDF.
- **Slide titles in the "On This Page" TOC.** Each slide's first heading now
  appears in mdBook's right-hand Table of Contents panel, like a normal
  chapter's subheadings. Clicking an entry jumps to its slide. Use `##` (h2) or
  deeper for titles you want listed (mdBook's panel shows h2–h6); heading ids
  match mdBook's `id_from_content` slug, so `[see X](slides.md#x)` deep-links
  work too. Identical titles are de-duplicated mdBook-style.

### Fixed
- **Dark-mode slide counter visibility.** The bottom-right `N / M` counter is now
  an explicit-color pill on a theme-agnostic translucent backing, so it stays
  legible on the dark mdBook themes (Coal/Navy/Ayu) where it previously washed
  out to near-invisible.
- **Firefox print/PDF hardening.** Firefox doesn't reliably honor CSS-driven
  page orientation (the portrait→landscape→portrait switch this book uses), so
  slide decks printed/exported from Firefox could come out portrait with
  overflowing content, collapsed columns, and stray blank pages. The print CSS
  now degrades gracefully — slides fill the page width and wrap within it
  regardless of orientation — and the README documents that Firefox users must
  select Landscape manually (Chrome/Edge do it automatically).

## [0.3.0] - 2026-07-03

### Added
- **Two-column slide layout.** Wrap content in `<div class="cols">` for a
  side-by-side layout; `cols-1-2` / `cols-2-1` weight one side. Columns stack on
  narrow screens and are re-asserted side-by-side in print.
- **Deep-linkable slides.** The active slide is mirrored into the URL as
  `#slide-N`; reloading or opening such a link lands on that slide, and the URL
  can be shared to point at a specific slide.

### Changed
- Slides are now **top-aligned** (heading at the top, content flowing beneath)
  instead of vertically centered. Section-divider slides — whose only content is
  a heading — remain centered. Layout metrics are exposed as `--slide-*` CSS
  custom properties so themes can override them without editing the shipped CSS.

### Compatibility
- No new version requirements — still targets mdBook 0.5. The switch to a
  top-aligned default layout is a visible change for existing decks, but it is
  not a breaking API change; themes can override the `--slide-*` properties to
  restore the old centred look.

### Fixed
- Print/PDF: tall slides now split across pages instead of being clipped
  (`page-break-inside: auto`), content no longer clips at the slide box
  (`overflow: visible`), long unbreakable tokens (URLs, paths, commands) wrap,
  and two-column slides stay two-column. The redundant chapter-title page before
  a deck is hidden.
- Code blocks no longer collapse to a single scrolling line inside the slide's
  flex column (`flex-shrink: 0`).
- A chapter that looks like a slides deck but whose frontmatter wasn't detected
  now logs a warning naming the chapter (mdBook 0.5 needs a blank line before
  the closing `---`), instead of silently rendering as prose. Documented the
  caveat in the README and fixed the bundled examples.

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

[Unreleased]: https://github.com/anantshri/mdbook-slides/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/anantshri/mdbook-slides/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/anantshri/mdbook-slides/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/anantshri/mdbook-slides/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/anantshri/mdbook-slides/releases/tag/v0.1.1
