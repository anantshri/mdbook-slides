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
                    mdbook_slides::html_template::render_presentation(&parsed.content);
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

    let html = mdbook_slides::html_template::render_presentation(&parsed.content);

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
    let html = mdbook_slides::html_template::render_presentation(&parsed.content);

    assert!(html.contains("<li>Item one</li>"));
    assert!(html.contains("let x = 42;"));
    assert!(html.contains("Final Slide"));
    assert!(html.contains("1 / 3"));
}
