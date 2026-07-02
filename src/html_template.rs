use pulldown_cmark::{html, Options, Parser};
use regex::Regex;
use std::sync::LazyLock;

static SLIDE_SEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^---[ \t]*$").unwrap());

/// Split markdown content into individual slides on `---` separators.
fn split_slides(markdown: &str) -> Vec<String> {
    SLIDE_SEP
        .split(markdown)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Strip speaker notes (everything after a line starting with `Note:` or `Notes:`).
fn strip_notes(slide: &str) -> &str {
    for prefix in ["\nNote:", "\nNotes:"] {
        if let Some(pos) = slide.find(prefix) {
            return slide[..pos].trim_end();
        }
    }
    slide
}

/// Render markdown to HTML using pulldown-cmark.
fn render_markdown(markdown: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(markdown, opts);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Replace blank lines with HTML comments to prevent CommonMark
/// from terminating the type-6 HTML block started by the outer `<div>`.
fn prevent_blank_lines(html: &str) -> String {
    html.lines()
        .map(|line| if line.trim().is_empty() { "<!-- -->" } else { line })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Generate the full HTML for a presentation chapter.
pub fn render_presentation(markdown: &str) -> String {
    let slides: Vec<String> = split_slides(markdown)
        .iter()
        .map(|s| {
            let content = strip_notes(s);
            render_markdown(content)
        })
        .collect();

    let total = slides.len();
    let mut out = String::new();

    // Slides container — starts a type 6 HTML block (no blank lines allowed)
    out.push_str("<div class=\"slides-container\">\n");
    for (i, slide_html) in slides.iter().enumerate() {
        let class = if i == 0 { "slide active" } else { "slide" };
        out.push_str(&format!("<div class=\"{class}\">\n"));
        out.push_str(&prevent_blank_lines(slide_html));
        out.push_str("</div>\n");
    }
    out.push_str(&format!(
        "<div class=\"slides-nav\"><span class=\"slides-progress\">1 / {total}</span></div>\n"
    ));
    out.push_str("</div>\n");

    // Blank line ends the type 6 block; <script> starts a type 1 block
    // (blank lines inside type 1 are fine)
    out.push_str(&format!("\n<script>\n{SLIDESHOW_JS}\n</script>"));

    out
}

const SLIDESHOW_JS: &str = r#"document.addEventListener('DOMContentLoaded', function() {
  var container = document.querySelector('.slides-container');
  var slides = container.querySelectorAll('.slide');
  var progress = container.querySelector('.slides-progress');
  var current = 0;

  // Save chapter nav URLs (absolute via .href) then remove the elements
  var nextUrl = null, prevUrl = null;
  var ne = document.querySelector('.nav-chapters.next');
  var pe = document.querySelector('.nav-chapters.previous');
  if (ne) nextUrl = ne.href;
  if (pe) prevUrl = pe.href;
  document.querySelectorAll('.nav-chapters').forEach(function(el) { el.remove(); });

  // Expand content area
  var m = document.querySelector('#content main') || document.querySelector('main');
  if (m) { m.style.padding = '0'; m.style.maxWidth = 'none'; }

  function show(n) {
    slides[current].classList.remove('active');
    current = n;
    slides[current].classList.add('active');
    progress.textContent = (current + 1) + ' / ' + slides.length;
    // Mirror the active slide into the URL so decks are deep-linkable.
    // replaceState avoids history spam and the scroll jump a bare hash set
    // would cause (there is no element with id="slide-N").
    history.replaceState(null, '', '#slide-' + (current + 1));
  }

  // Choose the starting slide: an explicit #slide-N in the URL wins (deep
  // link / reload); otherwise, if navigating backward from the next chapter,
  // start at the last slide.
  var referrer = document.referrer.replace(/#.*/, '');
  var hashMatch = /^#slide-(\d+)$/.exec(location.hash);
  if (hashMatch) {
    var idx = Math.min(Math.max(parseInt(hashMatch[1], 10), 1), slides.length) - 1;
    if (idx !== 0) show(idx);
  } else if (nextUrl && referrer === nextUrl.replace(/#.*/, '')) {
    show(slides.length - 1);
  }

  document.addEventListener('keydown', function(e) {
    if (e.key === 'ArrowRight' || e.key === 'ArrowDown' || e.key === ' ') {
      if (current < slides.length - 1) { show(current + 1); e.preventDefault(); }
      else if (nextUrl) window.location.href = nextUrl;
    } else if (e.key === 'ArrowLeft' || e.key === 'ArrowUp') {
      if (current > 0) { show(current - 1); e.preventDefault(); }
      else if (prevUrl) window.location.href = prevUrl;
    }
  });
});"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_slides() {
        let md = "# Slide 1\n\n---\n\n# Slide 2\n\n---\n\n# Slide 3";
        let slides = split_slides(md);
        assert_eq!(slides.len(), 3);
        assert_eq!(slides[0], "# Slide 1");
        assert_eq!(slides[1], "# Slide 2");
        assert_eq!(slides[2], "# Slide 3");
    }

    #[test]
    fn test_split_slides_no_separator() {
        let md = "# Only One Slide\n\nContent here.";
        let slides = split_slides(md);
        assert_eq!(slides.len(), 1);
    }

    #[test]
    fn test_strip_notes() {
        assert_eq!(strip_notes("Content\n\nNote:\nSpeaker notes"), "Content");
        assert_eq!(strip_notes("Content\n\nNotes:\nSpeaker notes"), "Content");
        assert_eq!(strip_notes("No notes here"), "No notes here");
    }

    #[test]
    fn test_render_markdown() {
        let html = render_markdown("# Hello\n\nWorld");
        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("<p>World</p>"));
    }

    #[test]
    fn test_prevent_blank_lines() {
        let input = "line1\n\nline3";
        let result = prevent_blank_lines(input);
        assert_eq!(result, "line1\n<!-- -->\nline3");
    }

    #[test]
    fn test_render_presentation_structure() {
        let html = render_presentation("# Slide 1\n\n---\n\n# Slide 2");
        assert!(html.starts_with("<div"));
        assert!(html.contains("class=\"slide active\""));
        assert!(html.contains("class=\"slide\""));
        assert!(html.contains("1 / 2"));
        assert!(html.contains("<h1>Slide 1</h1>"));
        assert!(html.contains("<h1>Slide 2</h1>"));
        // The embedded navigation script deep-links the active slide.
        assert!(html.contains("#slide-"));
        assert!(html.contains("replaceState"));
    }

    #[test]
    fn test_slideshow_js_deeplink() {
        // Write side: the active slide is mirrored into the URL.
        assert!(SLIDESHOW_JS.contains("replaceState"));
        assert!(SLIDESHOW_JS.contains("'#slide-' + (current + 1)"));
        // Read side: an explicit #slide-N in the URL sets the starting slide.
        assert!(SLIDESHOW_JS.contains("location.hash"));
        assert!(SLIDESHOW_JS.contains("#slide-(\\d+)"));
    }

    #[test]
    fn test_render_presentation_no_blank_lines_in_div_block() {
        let html = render_presentation(
            "# Slide 1\n\nParagraph\n\n---\n\n# Slide 2\n\n```\ncode\n\nmore\n```",
        );
        // The div block (before the script) must have no blank lines
        let div_block = html.split("\n\n<script>").next().unwrap();
        for (i, line) in div_block.lines().enumerate() {
            assert!(
                !line.trim().is_empty(),
                "Blank line at line {}: {:?}",
                i + 1,
                line
            );
        }
    }

    #[test]
    fn test_speaker_notes_stripped() {
        let html = render_presentation("Content\n\nNote:\nSecret notes\n\n---\n\n# Slide 2");
        assert!(!html.contains("Secret notes"));
        assert!(html.contains("Content"));
    }
}
