use pulldown_cmark::{html, Event, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static SLIDE_SEP: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^---[ \t]*$").unwrap());

/// Matches an opening heading tag `<h1>`–`<h6>` (optionally preceded by leading
/// whitespace) at the very start of a rendered slide, so we can locate the
/// slide's first heading and inject mdBook-style id + anchor. We match only the
/// opening tag (the Rust `regex` crate has no backreferences) and then find the
/// corresponding `</hN>` programmatically. Anchored to the start because a
/// slide's first child is its title heading when present; a slide that opens
/// with prose won't match and correctly gets no anchor.
static FIRST_HEADING_OPEN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A\s*<h([1-6])>").unwrap());

/// Reproduces mdBook's `id_from_content` (mdbook-html 0.5.4, utils.rs).
///
/// `trim()` + `to_lowercase()`, then per char: keep `is_alphanumeric()`,
/// `'_'`, or `'-'`; map whitespace to `'-'`; drop everything else. No collapse
/// of consecutive hyphens; Unicode letters/digits are preserved. Rust's
/// `char::is_alphanumeric()` is Unicode-aware, matching mdBook exactly.
fn id_from_content(content: &str) -> String {
    content
        .trim()
        .to_lowercase()
        .chars()
        .filter_map(|ch| {
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                Some(ch)
            } else if ch.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

/// Reproduces mdBook's `unique_id` (mdbook-html 0.5.4, utils.rs): if `id` is
/// already in `used`, append `-1`, `-2`, … until unused (recording each winner).
fn unique_id(id: &str, used: &mut HashSet<String>) -> String {
    if used.insert(id.to_string()) {
        return id.to_string();
    }
    let mut counter: u32 = 1;
    loop {
        let candidate = format!("{id}-{counter}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}

/// The first heading found in a slide's markdown: its level (1–6) and the
/// concatenated plain-text content of its inline children (`Text`/`Code`
/// events), matching mdBook's `text_in_node`. Enough to build a slug; the
/// rendered HTML keeps rich children intact for the anchor label.
struct FirstHeading {
    level: u8,
    text: String,
}

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

/// Render markdown to HTML using pulldown-cmark. (Used only by tests; the deck
/// renderer goes through `render_markdown_with_heading` to also capture the
/// slide's first heading for TOC anchoring.)
#[cfg(test)]
fn render_markdown(markdown: &str) -> String {
    render_markdown_with_heading(markdown).0
}

/// Render slide markdown to HTML in a single pulldown-cmark pass, and also
/// capture the slide's FIRST heading (level + plain-text inner content) so the
/// caller can inject an mdBook-style anchor for the right-hand "On This Page"
/// TOC. Headings buried inside a raw HTML block (CommonMark type-6) are not
/// emitted as `Tag::Heading` by pulldown-cmark, so they are correctly ignored —
/// the slide's first *markdown* heading wins.
fn render_markdown_with_heading(markdown: &str) -> (String, Option<FirstHeading>) {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(markdown, opts);

    // Single pass: collect events for rendering while capturing the first
    // heading's level + plain-text content. We keep the events around so we can
    // hand them to push_html after the loop (push_html consumes the iterator).
    let mut events: Vec<Event> = Vec::new();
    let mut first_heading: Option<FirstHeading> = None;
    let mut buf: Option<String> = None; // buffering only while inside the target heading
    let mut open_level: Option<u8> = None;

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. })
                if first_heading.is_none() && buf.is_none() =>
            {
                open_level = Some(level_to_u8(level));
                buf = Some(String::new());
            }
            Event::Text(ref s) | Event::Code(ref s) if open_level.is_some() => {
                if let Some(b) = buf.as_mut() {
                    b.push_str(s.as_ref());
                }
            }
            Event::End(TagEnd::Heading(level)) if open_level.is_some() => {
                let level_u = level_to_u8(level);
                if open_level == Some(level_u) {
                    if let Some(text) = buf.take() {
                        first_heading = Some(FirstHeading { level: level_u, text });
                    }
                    open_level = None;
                }
            }
            _ => {}
        }
        events.push(event);
    }

    let mut html_output = String::new();
    html::push_html(&mut html_output, events.into_iter());

    (html_output, first_heading)
}

fn level_to_u8(level: pulldown_cmark::HeadingLevel) -> u8 {
    use pulldown_cmark::HeadingLevel::*;
    match level {
        H1 => 1,
        H2 => 2,
        H3 => 3,
        H4 => 4,
        H5 => 5,
        H6 => 6,
    }
}

/// If `html` opens with an `<hN>…</hN>` heading, inject mdBook's final heading
/// shape — `<hN id="slug"><a class="header" href="#slug">…inner…</a></hN>` — so
/// mdBook's `toc.js` (which selects h2–h6 having a non-empty id whose first
/// child is an `<a>`) lists it in the right-hand "On This Page" panel. Inner
/// rich children (e.g. `<code>`) are preserved verbatim inside the anchor,
/// matching mdBook's `add_header_links`. Returns the input unchanged when there
/// is no leading heading. Run BEFORE `prevent_blank_lines` so the regex sees
/// normal HTML; the added markup is single-line, so the type-6-block no-blank-
/// line invariant still holds.
fn anchorize_first_heading(html: &str, slug: &str) -> String {
    // Locate the leading `<hN>` open tag; the Rust `regex` crate has no
    // backreferences, so we capture only the level and find `</hN>` by hand.
    let Some(caps) = FIRST_HEADING_OPEN_RE.captures(html) else {
        return html.to_string();
    };
    let level = caps[1].to_string(); // "1".."6"
    let open_match = caps.get(0).unwrap();
    let open_end = open_match.end();

    let close = format!("</h{level}>");
    let Some(rel) = html[open_end..].find(&close) else {
        return html.to_string();
    };
    let inner = &html[open_end..open_end + rel];
    let full_end = open_end + rel + close.len();

    let mut out = String::with_capacity(html.len() + slug.len() * 2 + 40);
    out.push_str(&html[..open_match.start()]);
    // <hN id="SLUG"><a class="header" href="#SLUG">INNER</a></hN>
    out.push_str("<h");
    out.push_str(&level);
    out.push_str(" id=\"");
    out.push_str(slug);
    out.push_str("\"><a class=\"header\" href=\"#");
    out.push_str(slug);
    out.push_str("\">");
    out.push_str(inner);
    out.push_str("</a></h");
    out.push_str(&level);
    out.push('>');
    out.push_str(&html[full_end..]);
    out
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
    // Render each slide once; for slides whose first heading is h2–h6, capture
    // that heading so we can inject the mdBook anchor that makes it appear in
    // the right-hand "On This Page" TOC. Slug ids are deduplicated across the
    // whole deck so identical titles (e.g. two "Demo" slides) get distinct,
    // mdBook-compatible anchors (demo, demo-1, …).
    let mut used_ids: HashSet<String> = HashSet::new();
    let slides: Vec<String> = split_slides(markdown)
        .iter()
        .map(|s| {
            let content = strip_notes(s);
            let (mut html, heading) = render_markdown_with_heading(content);
            if let Some(h) = heading {
                if h.level >= 2 {
                    let slug = id_from_content(&h.text);
                    let slug = unique_id(&slug, &mut used_ids);
                    html = anchorize_first_heading(&html, &slug);
                }
            }
            html
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
  // link / reload); a heading-slug like #features (e.g. clicked in mdBook's
  // right-hand "On This Page" TOC) reveals the slide containing that heading;
  // otherwise, if navigating backward from the next chapter, start at the last
  // slide.
  var referrer = document.referrer.replace(/#.*/, '');
  var hashMatch = /^#slide-(\d+)$/.exec(location.hash);
  if (hashMatch) {
    var idx = Math.min(Math.max(parseInt(hashMatch[1], 10), 1), slides.length) - 1;
    if (idx !== 0) show(idx);
  } else if (location.hash) {
    // A TOC deep link: locate the heading by id and reveal the slide it lives in.
    var heading = document.getElementById(location.hash.slice(1));
    if (heading) {
      var slideEl = heading.closest('.slide');
      if (slideEl) {
        var idx = Array.prototype.indexOf.call(slides, slideEl);
        if (idx >= 0 && idx !== current) show(idx);
      }
    }
  } else if (nextUrl && referrer === nextUrl.replace(/#.*/, '')) {
    show(slides.length - 1);
  }

  // Clicking a right-hand "On This Page" TOC entry changes the URL hash to the
  // heading's slug mid-session; reveal the slide containing that heading. The
  // existing #slide-N form (set by show() via replaceState, which fires no
  // hashchange) is handled first.
  window.addEventListener('hashchange', function() {
    var hash = location.hash;
    var m = /^#slide-(\d+)$/.exec(hash);
    if (m) {
      var idx = Math.min(Math.max(parseInt(m[1], 10), 1), slides.length) - 1;
      if (idx !== current) show(idx);
      return;
    }
    if (hash) {
      var heading = document.getElementById(hash.slice(1));
      if (heading) {
        var slideEl = heading.closest('.slide');
        if (slideEl) {
          var idx = Array.prototype.indexOf.call(slides, slideEl);
          if (idx >= 0 && idx !== current) show(idx);
        }
      }
    }
  });

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

    // ---- slug + anchor (per-slide "On This Page" TOC) ----

    #[test]
    fn test_id_from_content_basic() {
        assert_eq!(id_from_content("Hello World"), "hello-world");
    }

    #[test]
    fn test_id_from_content_mdbook_vectors() {
        // Exact vectors from mdbook-html 0.5.4/src/utils.rs it_normalizes_ids.
        assert_eq!(
            id_from_content("`--passes`: add more rustdoc passes"),
            "--passes-add-more-rustdoc-passes"
        );
        assert_eq!(
            id_from_content("Method-call 🐙 expressions \u{1f47c}"),
            "method-call--expressions-"
        );
        assert_eq!(id_from_content("_-_12345"), "_-_12345");
        assert_eq!(id_from_content("12345"), "12345");
        assert_eq!(id_from_content("中文"), "中文");
        assert_eq!(id_from_content("にほんご"), "にほんご");
        assert_eq!(id_from_content("한국어"), "한국어");
        assert_eq!(id_from_content(""), "");
        assert_eq!(id_from_content("中文標題 CJK title"), "中文標題-cjk-title");
        assert_eq!(id_from_content("Über"), "über");
    }

    #[test]
    fn test_id_from_content_consecutive_hyphens_no_collapse() {
        // Two spaces -> two hyphens (mdBook does not collapse them).
        assert_eq!(id_from_content("A  B"), "a--b");
    }

    #[test]
    fn test_id_from_content_trailing_trim_then_drop() {
        // trim() runs first, so trailing whitespace is removed before any char
        // is mapped to '-'. A trailing space thus yields no trailing hyphen.
        assert_eq!(id_from_content("A "), "a");
        // But a trailing non-alphanumeric, non-whitespace char (e.g. an emoji)
        // survives trim, and the preceding space maps to '-', giving a trailing '-'.
        assert_eq!(id_from_content("A \u{1f47c}"), "a-");
    }

    #[test]
    fn test_unique_id_first_use() {
        let mut used = HashSet::new();
        assert_eq!(unique_id("features", &mut used), "features");
        assert_eq!(used.len(), 1);
    }

    #[test]
    fn test_unique_id_dedup() {
        let mut used = HashSet::new();
        assert_eq!(unique_id("features", &mut used), "features");
        assert_eq!(unique_id("features", &mut used), "features-1");
        assert_eq!(unique_id("features", &mut used), "features-2");
    }

    #[test]
    fn test_render_markdown_with_heading_h2() {
        let (html, h) = render_markdown_with_heading("## Features\n\nbullets");
        assert!(html.contains("<h2>Features</h2>"));
        let h = h.expect("heading captured");
        assert_eq!(h.level, 2);
        assert_eq!(h.text, "Features");
    }

    #[test]
    fn test_render_markdown_with_heading_inline_code() {
        // Inline code in a heading: slug text is flattened, rich child preserved in HTML.
        let (html, h) = render_markdown_with_heading("## Code `x` Example");
        assert!(html.contains("<h2>Code <code>x</code> Example</h2>"));
        let h = h.expect("heading captured");
        assert_eq!(h.level, 2);
        assert_eq!(h.text, "Code x Example");
    }

    #[test]
    fn test_render_markdown_with_heading_no_heading() {
        let (_html, h) = render_markdown_with_heading("Just prose\n");
        assert!(h.is_none());
    }

    #[test]
    fn test_render_markdown_with_heading_levels() {
        // Exercise heading-level capture across h3–h6 (drives the level_to_u8 arms).
        for (md, want) in [
            ("### Three", 3u8),
            ("#### Four", 4),
            ("##### Five", 5),
            ("###### Six", 6),
        ] {
            let (_html, h) = render_markdown_with_heading(md);
            assert_eq!(h.expect("heading captured").level, want, "input {md:?}");
        }
    }

    #[test]
    fn test_render_markdown_with_heading_picks_first() {
        // Only the FIRST heading is captured for the slug; later headings still render.
        let (html, h) = render_markdown_with_heading("## A\n\n### B\n");
        assert!(html.contains("<h2>A</h2>"));
        assert!(html.contains("<h3>B</h3>"));
        let h = h.expect("heading captured");
        assert_eq!(h.level, 2);
        assert_eq!(h.text, "A");
    }

    #[test]
    fn test_anchorize_first_heading_basic() {
        let out = anchorize_first_heading("<h2>Features</h2>", "features");
        assert_eq!(
            out,
            r##"<h2 id="features"><a class="header" href="#features">Features</a></h2>"##
        );
    }

    #[test]
    fn test_anchorize_first_heading_preserves_inner() {
        let out = anchorize_first_heading("<h2>Code <code>x</code> Example</h2>", "code-x-example");
        assert_eq!(
            out,
            r##"<h2 id="code-x-example"><a class="header" href="#code-x-example">Code <code>x</code> Example</a></h2>"##
        );
    }

    #[test]
    fn test_anchorize_first_heading_no_heading() {
        // No leading heading -> unchanged.
        let out = anchorize_first_heading("<p>no heading</p>", "x");
        assert_eq!(out, "<p>no heading</p>");
    }

    #[test]
    fn test_anchorize_first_heading_preserves_trailing_content() {
        // Heading not at the very start/end: surrounding HTML survives.
        let out = anchorize_first_heading("<h2>Hi</h2>\n<p>body</p>", "hi");
        assert!(out.starts_with(r##"<h2 id="hi"><a class="header" href="#hi">Hi</a></h2>"##));
        assert!(out.ends_with("\n<p>body</p>"));
    }

    #[test]
    fn test_render_presentation_anchors_first_heading() {
        let html = render_presentation("## Welcome\n\n---\n\n## Features");
        assert!(html.contains(
            r##"<h2 id="welcome"><a class="header" href="#welcome">Welcome</a></h2>"##
        ));
        assert!(html.contains(
            r##"<h2 id="features"><a class="header" href="#features">Features</a></h2>"##
        ));
    }

    #[test]
    fn test_render_presentation_no_anchor_for_headingless_slide() {
        let html = render_presentation("## First\n\n---\n\nJust prose, no heading");
        assert!(html.contains(r#"<h2 id="first">"#));
        // The prose slide has no heading, so it must contribute no anchored <hN id=...>.
        assert!(html.contains("<p>Just prose, no heading</p>"));
        // Only one anchored heading in the whole deck.
        assert_eq!(html.matches("class=\"header\"").count(), 1);
    }

    #[test]
    fn test_render_presentation_dedups_identical_heading_text() {
        let html = render_presentation("## Features\n\n---\n\n## Features");
        assert!(html.contains(r#"<h2 id="features">"#));
        assert!(html.contains(r#"<h2 id="features-1">"#));
    }

    #[test]
    fn test_render_presentation_subheadings_not_anchored() {
        // Only the first heading per slide is anchored; the sub-heading stays bare
        // so it does not leak into the right-hand TOC.
        let html = render_presentation("## Top\n\n### Sub\n");
        assert!(html.contains(r##"<h2 id="top"><a class="header" href="#top">Top</a></h2>"##));
        assert!(html.contains("<h3>Sub</h3>"));
        assert!(!html.contains(r#"<h3 id="#));
    }

    #[test]
    fn test_render_presentation_h1_not_anchored() {
        // toc.js only selects h2-h6, so h1 slides stay bare (no dead anchor).
        let html = render_presentation("# Solo H1\n");
        assert!(html.contains("<h1>Solo H1</h1>"));
        assert!(!html.contains("class=\"header\""));
    }

    #[test]
    fn test_render_presentation_anchored_block_has_no_blank_lines() {
        // The no-blank-lines invariant must survive the anchor injection.
        let html = render_presentation("## Welcome\n\nIntro\n\n---\n\n## Features\n\n```\ncode\n```");
        let div_block = html.split("\n\n<script>").next().unwrap();
        assert!(div_block.contains(r#"<h2 id="welcome">"#));
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
    fn test_slideshow_js_toc_deep_link() {
        // On load: heading-slug deep link resolves to the slide containing it.
        assert!(SLIDESHOW_JS.contains("getElementById(location.hash.slice(1))"));
        assert!(SLIDESHOW_JS.contains("heading.closest('.slide')"));
        assert!(SLIDESHOW_JS.contains("Array.prototype.indexOf.call(slides"));
        // Mid-session TOC click: a hashchange listener reveals the slide.
        assert!(SLIDESHOW_JS.contains("addEventListener('hashchange'"));
    }
}
