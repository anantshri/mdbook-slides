use anyhow::Result;
use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;

static FRONTMATTER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\A---[ \t]*\n([\s\S]*?)\n---[ \t]*\n").unwrap());

/// Per-file frontmatter configuration parsed from YAML.
/// Unknown fields are silently ignored.
#[derive(Debug, Default, Deserialize)]
pub struct FrontmatterConfig {
    #[serde(default)]
    pub slides: bool,
}

/// Result of parsing frontmatter from a markdown file.
pub struct ParseResult {
    pub config: FrontmatterConfig,
    /// The markdown content after stripping the frontmatter block.
    pub content: String,
}

/// Parse YAML frontmatter from markdown content.
///
/// Returns the parsed config and remaining content. If no frontmatter is found,
/// returns a default config and the original content unchanged.
pub fn parse_frontmatter(input: &str) -> Result<ParseResult> {
    if let Some(caps) = FRONTMATTER_RE.captures(input) {
        let yaml_str = &caps[1];
        let config: FrontmatterConfig = serde_yaml_ng::from_str(yaml_str)?;
        let content = input[caps.get(0).unwrap().end()..].to_string();
        Ok(ParseResult { config, content })
    } else {
        Ok(ParseResult {
            config: FrontmatterConfig::default(),
            content: input.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slides_frontmatter() {
        let input = "---\nslides: true\ntheme: solarized\n---\n# Slide 1\n---\n# Slide 2\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(result.config.slides);
        assert_eq!(result.content, "# Slide 1\n---\n# Slide 2\n");
    }

    #[test]
    fn test_parse_frontmatter_blank_line_before_close() {
        // The mdBook-0.5-safe shape (blank line before the closing `---`) must
        // still parse: the blank line is part of the YAML block, not a terminator.
        let input = "---\nslides: true\n\n---\n# Slide 1\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(result.config.slides);
        assert_eq!(result.content, "# Slide 1\n");
    }

    #[test]
    fn test_no_frontmatter() {
        let input = "# Just markdown\n---\nMore content\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(!result.config.slides);
        assert_eq!(result.content, input);
    }

    #[test]
    fn test_slide_separator_not_confused_with_frontmatter() {
        let input = "Some text\n---\nMore text\n---\nEven more\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(!result.config.slides);
        assert_eq!(result.content, input);
    }

    #[test]
    fn test_unknown_frontmatter_fields_ignored() {
        let input = "---\nslides: true\ntheme: white\nrevealOptions:\n  transition: fade\n---\nContent\n";
        let result = parse_frontmatter(input).unwrap();
        assert!(result.config.slides);
        assert_eq!(result.content, "Content\n");
    }
}
