use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::Preprocessor;

#[test]
fn test_mixed_book_slides_and_normal() {
    let slides_content =
        "---\nslides: true\ntheme: white\n---\n## Slide 1\n\n---\n\n## Slide 2\n\n---\n\n## Slide 3\n";
    let normal_content = "# Regular Chapter\n\nThis is normal markdown content.\n";

    let mut book = Book::new();
    book.push_item(BookItem::Chapter(Chapter::new(
        "My Slides",
        slides_content.to_string(),
        "slides.md",
        Vec::new(),
    )));
    book.push_item(BookItem::Chapter(Chapter::new(
        "Regular Content",
        normal_content.to_string(),
        "content.md",
        Vec::new(),
    )));

    // Process each chapter manually
    book.for_each_mut(|item| {
        if let BookItem::Chapter(ref mut ch) = item {
            let parsed = mdbook_slides::frontmatter::parse_frontmatter(&ch.content).unwrap();
            if parsed.config.slides {
                ch.content =
                    mdbook_slides::html_template::render_presentation(&parsed.content, &ch.name);
            }
        }
    });

    // Verify slides chapter was transformed
    let mut items = book.iter();
    if let Some(BookItem::Chapter(slides)) = items.next() {
        assert!(slides.content.starts_with("<div"), "Should start with HTML div");
        assert!(slides.content.contains("Slide 1"), "Should preserve slide content");
        assert!(slides.content.contains("Slide 3"), "Should preserve all slides");
        assert!(!slides.content.contains("slides: true"), "Should strip frontmatter");
        // Navigation zones and the chapter-name orientation label are present.
        assert!(slides.content.contains("class=\"slides-zone next\""), "Should render nav zones");
        assert!(
            slides.content.contains("<div class=\"slides-chapter\"><span>My Slides</span></div>"),
            "Should render the chapter-name label"
        );
    } else {
        panic!("First item should be a chapter");
    }

    // Verify normal chapter was left unchanged
    if let Some(BookItem::Chapter(normal)) = items.next() {
        assert_eq!(normal.content, normal_content, "Normal chapter should be unchanged");
    } else {
        panic!("Second item should be a chapter");
    }
}

#[test]
fn test_preprocessor_name_and_renderer() {
    let preprocessor = mdbook_slides::SlidesPreprocessor;
    assert_eq!(preprocessor.name(), "slides");
    assert!(preprocessor.supports_renderer("html").unwrap());
    assert!(!preprocessor.supports_renderer("latex").unwrap());
}

#[test]
fn test_slides_with_code_and_notes() {
    let content = "---\nslides: true\n---\n## Code\n\n```rust\nfn main() {}\n```\n\nNote:\nSecret notes\n\n---\n\n## End\n";

    let parsed = mdbook_slides::frontmatter::parse_frontmatter(content).unwrap();
    assert!(parsed.config.slides);

    let html = mdbook_slides::html_template::render_presentation(&parsed.content, "Deck");

    assert!(html.contains("fn main()"));
    assert!(!html.contains("Secret notes"));
    assert!(html.contains("End"));
}

#[test]
fn test_standard_markdown_features_preserved() {
    let content = r#"---
slides: true
---
## Slide 1

- Item one
- Item two
- Item three

---

## Code Example

```rust
let x = 42;
```

---

## Final Slide
"#;

    let parsed = mdbook_slides::frontmatter::parse_frontmatter(content).unwrap();
    let html = mdbook_slides::html_template::render_presentation(&parsed.content, "Deck");

    assert!(html.contains("<li>Item one</li>"));
    assert!(html.contains("let x = 42;"));
    assert!(html.contains("Final Slide"));
    assert!(html.contains("1 / 3"));
}

#[test]
fn test_slides_appear_in_toc_markup() {
    // Each slide's first heading is emitted in mdBook's final anchored shape so
    // mdBook's toc.js lists it in the right-hand "On This Page" panel. Slug ids
    // match mdBook's id_from_content, and identical titles are deduplicated.
    let content = "---\nslides: true\n---\n## Welcome\n\n---\n\n## Features\n\n---\n\n## Welcome\n";
    let parsed = mdbook_slides::frontmatter::parse_frontmatter(content).unwrap();
    let html = mdbook_slides::html_template::render_presentation(&parsed.content, "Deck");

    // mdBook shape: <h2 id="slug"><a class="header" href="#slug">Text</a></h2>
    assert!(html.contains(r##"<h2 id="welcome"><a class="header" href="#welcome">Welcome</a></h2>"##));
    assert!(html.contains(r##"<h2 id="features"><a class="header" href="#features">Features</a></h2>"##));
    // Duplicate "Welcome" title deduplicated mdBook-style.
    assert!(html.contains(r##"<h2 id="welcome-1"><a class="header" href="#welcome-1">Welcome</a></h2>"##));
}
