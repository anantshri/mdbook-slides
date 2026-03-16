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

### Markdown Features

Standard markdown works inside slides:

- Headings, paragraphs, lists
- Code blocks with syntax highlighting (via mdbook's built-in highlighter)
- Tables, images, links
- Speaker notes (lines after `Note:` or `Notes:` are stripped from output)

## Print / PDF

The included print CSS ensures presentations render properly when printed or exported to PDF:

- Each slide becomes its own **landscape** page
- Regular chapters remain **portrait**
- Speaker notes hidden
- Navigation controls hidden
- Background colors preserved

Just use your browser's print function or mdbook's print page (`/print.html`).

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
