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
        return Ok(());
    }

    log::info!("Processing slides chapter: {}", chapter.name);

    let html = html_template::render_presentation(&parsed.content);
    chapter.content = html;

    Ok(())
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
}
