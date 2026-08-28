pub mod frontmatter;
pub mod html_template;
pub mod install;

use anyhow::Result;
use mdbook_preprocessor::book::{Book, BookItem, Chapter};
use mdbook_preprocessor::{Preprocessor, PreprocessorContext};

pub struct SlidesPreprocessor;

impl Preprocessor for SlidesPreprocessor {
    fn name(&self) -> &str {
        "slides"
    }

    fn supports_renderer(&self, renderer: &str) -> Result<bool> {
        Ok(renderer == "html")
    }

    fn run(&self, _ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        book.for_each_mut(|item| {
            if let BookItem::Chapter(ref mut chapter) = item {
                if let Err(e) = process_chapter(chapter) {
                    log::error!("Error processing chapter '{}': {}", chapter.name, e);
                }
            }
        });

        Ok(book)
    }
}

fn process_chapter(chapter: &mut Chapter) -> Result<()> {
    let parsed = frontmatter::parse_frontmatter(&chapter.content)?;

    if !parsed.config.slides {
        // mdBook 0.5 round-trips chapter markdown through pulldown-cmark before
        // preprocessors run. If the frontmatter's closing `---` directly follows
        // the last key (no blank line), CommonMark's setext rule turns
        // `slides: true\n---` into a heading and consumes the closing marker, so
        // the frontmatter never parses and the deck silently renders as prose.
        // Warn when a chapter looks like it *intended* slides frontmatter.
        if looks_like_mangled_slides_frontmatter(&chapter.content) {
            log::warn!(
                "Chapter '{}' looks like a slides deck but its frontmatter was not \
                 detected. On mdBook 0.5, add a blank line before the closing `---` \
                 of the frontmatter block (see the README, \"Frontmatter\").",
                chapter.name
            );
        }
        return Ok(());
    }

    log::info!("Processing slides chapter: {}", chapter.name);

    let html = html_template::render_presentation(&parsed.content, &chapter.name);
    chapter.content = html;

    Ok(())
}

/// Heuristic: does this chapter look like it *tried* to declare `slides: true`
/// frontmatter but had it mangled (e.g. by mdBook's markdown round-trip eating
/// the closing `---`)? Only the top of the file is inspected so body prose that
/// merely mentions the phrase does not trip a false warning.
fn looks_like_mangled_slides_frontmatter(content: &str) -> bool {
    let head = content.chars().take(200).collect::<String>().to_ascii_lowercase();
    head.contains("slides: true") || head.contains("slides:true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_slides_chapter() {
        let mut chapter = Chapter::new(
            "Test",
            "---\nslides: true\n---\n# Slide 1\n\n---\n\n# Slide 2\n".to_string(),
            "test.md",
            Vec::new(),
        );

        process_chapter(&mut chapter).unwrap();

        assert!(chapter.content.starts_with("<div"));
        assert!(chapter.content.contains("Slide 1"));
        assert!(chapter.content.contains("Slide 2"));
    }

    #[test]
    fn test_skip_non_slides_chapter() {
        let original = "# Just a regular chapter\nWith content.".to_string();
        let mut chapter = Chapter::new("Normal", original.clone(), "normal.md", Vec::new());

        process_chapter(&mut chapter).unwrap();

        assert_eq!(chapter.content, original);
    }

    #[test]
    fn test_mangled_frontmatter_heuristic() {
        // Setext-mangled frontmatter (closing --- eaten) still carries the phrase.
        assert!(looks_like_mangled_slides_frontmatter("slides: true\n---\n# Deck"));
        assert!(looks_like_mangled_slides_frontmatter("## slides: true\n\n# Deck"));
        // Well-formed frontmatter and ordinary prose do not trip it.
        assert!(!looks_like_mangled_slides_frontmatter("# Just a chapter\n\nProse."));
        // The phrase deep in body text (past the inspected head) is ignored.
        let mut body = "x".repeat(300);
        body.push_str("slides: true");
        assert!(!looks_like_mangled_slides_frontmatter(&body));
    }
}
