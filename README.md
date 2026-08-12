# mdbook-slides

An [mdbook](https://rust-lang.github.io/mdBook/) preprocessor that turns markdown files into slide presentations — no JavaScript frameworks, no CDN, no dependencies beyond mdbook itself.

- One markdown file = one sidebar entry = a full-page slideshow
- Arrow keys navigate slides, then seamlessly continue to the next chapter
- Slides print as landscape pages; regular chapters stay portrait
- Zero client-side dependencies — everything is self-contained

## Quick Start

### Install

```sh
cargo install mdbook-slides
```

Or build from source:

```sh
cargo install --path .
```

### Add to your book

Run the install command in your book's root directory:

```sh
mdbook-slides install .
```

This will:
- Add `[preprocessor.slides]` to your `book.toml`
- Write slideshow and print CSS files to `src/css/`
- Register the CSS in `[output.html].additional-css`

### Create a presentation

Add a markdown file with `slides: true` in the YAML frontmatter. Separate slides with `---`:

````markdown
---
slides: true

---
## Welcome

Hello, world!

---

## Key Points

- First point
- Second point
- Third point

---

## Code Example

```rust
fn main() {
    println!("Hello from a slide!");
}
```

Note:
These are speaker notes — hidden from output.

---

## Thank You!

Questions?
````

> **mdBook 0.5 frontmatter caveat.** mdBook 0.5 round-trips chapter markdown
> through pulldown-cmark before preprocessors run. If the closing `---` of the
> frontmatter directly follows the last key, CommonMark's setext rule reads
> `slides: true` + `---` as a heading and consumes the closing marker, so the
> deck silently renders as ordinary prose. **Put a blank line before the closing
> `---`:**
>
> ```markdown
> ---
> slides: true
>
> ---
> ## First slide
> ```
>
> If a deck renders as a normal page, this is almost always the cause —
> mdbook-slides prints a warning naming the chapter when it detects it.

Reference it from `SUMMARY.md` like any other chapter:

```markdown
- [Introduction](intro.md)
- [My Presentation](slides.md)
- [Next Chapter](chapter.md)
```

Build as usual:

```sh
mdbook build
```

## How It Works

The Rust preprocessor:

1. Detects `slides: true` in YAML frontmatter
2. Splits the markdown on `---` separators into individual slides
3. Renders each slide's markdown to HTML (via pulldown-cmark)
4. Wraps slides in a minimal slideshow container with ~30 lines of vanilla JS

No external runtime, no CDN, no framework. The slideshow is pure HTML/CSS/JS embedded in the page.

### Navigation

| Key | Action |
|-----|--------|
| Right / Down / Space | Next slide |
| Left / Up | Previous slide |
| Right on last slide | Navigate to next chapter |
| Left on first slide | Navigate to previous chapter |

Navigation is direction-aware: arriving from the next chapter starts at the **last** slide, so you can seamlessly walk backward through the deck.

The active slide is reflected in the URL as `#slide-N` (1-based), so a deck is **deep-linkable** — copy the URL to share a specific slide, and reloading (or opening a `#slide-N` link) lands on that slide instead of the first.

### Table of Contents

Each slide's first heading is listed in mdBook's right-hand **"On This Page"**
panel, just like the subheadings of a normal chapter. Clicking an entry jumps
straight to the slide containing that heading (the URL becomes
`slides.html#heading-slug`, and the slide is revealed even though non-active
slides are hidden).

A few details:

- Use `##` (h2) or deeper for slide titles you want in the TOC — mdBook's panel
  lists `h2`–`h6` only, so a slide whose title is a single `#` (h1) won't get an
  entry.
- Only each slide's **first** heading becomes a TOC entry; additional headings on
  the same slide stay on the slide but don't clutter the panel.
- A slide with no heading simply gets no entry.
- Identical titles are de-duplicated mdBook-style (`welcome`, `welcome-1`, …).

This works because the preprocessor emits each slide's first heading in mdBook's
final anchored shape (`<h2 id="slug"><a class="header" href="#slug">…</a></h2>`),
which mdBook's TOC builder recognizes. Heading ids match mdBook's own
`id_from_content` slug, so you can also deep-link from elsewhere in the book
(`[see Features](slides.md#features)`).

### Markdown Features

Standard markdown works inside slides:

- Headings, paragraphs, lists
- Code blocks with syntax highlighting (via mdbook's built-in highlighter)
- Tables, images, links
- Speaker notes (lines after `Note:` or `Notes:` are stripped from output)

### Two-Column Layout

For side-by-side content, wrap two `<div>`s in a `<div class="cols">`:

````markdown
<div class="cols">
<div>

**Left column**

- bullets, text, code

</div>
<div>

**Right column**

![diagram](diagram.svg)

</div>
</div>
````

Leave a blank line around the inner content so CommonMark still parses it as
markdown (a raw `<div>` block runs until the next blank line). The default is an
even split; use `class="cols cols-1-2"` or `class="cols cols-2-1"` to weight one
side. Columns stack to a single column on narrow screens and are re-asserted
side-by-side when printing.

## Print / PDF

The included print CSS ensures presentations render properly when printed or exported to PDF:

- Each slide becomes its own **landscape** page
- Regular chapters remain **portrait**
- A slide taller than the page splits across pages instead of being clipped
- Long URLs, paths, and command lines wrap rather than running off the edge
- Speaker notes hidden
- Navigation controls hidden
- Background colors preserved

Just use your browser's print function or mdbook's print page (`/print.html`).

> **Firefox note.** Slides are designed for **landscape** pages. Chrome and
> Edge honor the CSS and print the deck landscape automatically. **Firefox does
> not reliably honor CSS-driven orientation** — especially the portrait →
> landscape → portrait switch this book uses — so in Firefox you must
> **manually select Landscape** in the print dialog (and even then a book that
> mixes slide decks with regular chapters may not switch orientation
> per-section). For clean PDF export, prefer **Chrome or Edge**. The print CSS
> degrades gracefully, so if Firefox falls back to portrait the content still
> wraps and stays within the page rather than being clipped.

## Project Structure

```
Cargo.toml                  # Binary: mdbook-slides
src/
  main.rs                   # CLI: stdin/stdout protocol, `supports`, `install` subcommands
  lib.rs                    # Preprocessor trait impl, chapter iteration
  frontmatter.rs            # YAML frontmatter parser (detect slides: true)
  html_template.rs          # Slide splitting, markdown rendering, HTML generation
  install.rs                # `install` subcommand: write assets, update book.toml
assets/
  slides.css                # Slideshow layout and styling
  slides-print.css          # Print/PDF: landscape slides, portrait chapters
tests/
  integration.rs            # End-to-end with Book structs
test-book/                  # Manual test book
```

## Development

```sh
cargo build                 # Compile
cargo test                  # Run all tests
```

To manually test with the included test book:

```sh
cd test-book && mdbook serve    # Live preview at localhost:3000
```

## AI-Assisted Development

This project was developed with the assistance of AI tools, primarily **Claude Code**. The AI assisted with architecture decisions, implementation, debugging, and iterative refinement across multiple sessions. All AI-generated code was reviewed and validated through human inspection and testing to ensure correctness and quality.

## License

GPL-3.0 — see [LICENSE](LICENSE) for details.
