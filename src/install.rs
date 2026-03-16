use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use toml_edit::{DocumentMut, Item, Table, Value};

const BRIDGE_CSS: &str = include_str!("../assets/slides.css");
const PRINT_CSS: &str = include_str!("../assets/slides-print.css");

/// Install the preprocessor into an mdbook project.
///
/// - Writes CSS asset files into the book's source directory
/// - Updates book.toml to register the preprocessor and additional CSS
pub fn install(book_dir: &Path) -> Result<()> {
    let toml_path = book_dir.join("book.toml");
    let toml_content = fs::read_to_string(&toml_path)
        .with_context(|| format!("Failed to read {}", toml_path.display()))?;

    let mut doc: DocumentMut = toml_content
        .parse()
        .context("Failed to parse book.toml")?;

    // Add [preprocessor.slides] if not present
    if doc.get("preprocessor").is_none() {
        doc["preprocessor"] = Item::Table(Table::new());
    }
    let preprocessor = doc["preprocessor"].as_table_mut().unwrap();
    if preprocessor.get("slides").is_none() {
        let mut slides = Table::new();
        slides.insert("command", Item::Value(Value::from("mdbook-slides")));
        preprocessor.insert("slides", Item::Table(slides));
    }

    // Determine source directory
    let src_dir = if let Some(build) = doc.get("book") {
        if let Some(src) = build.get("src") {
            book_dir.join(src.as_str().unwrap_or("src"))
        } else {
            book_dir.join("src")
        }
    } else {
        book_dir.join("src")
    };

    // Write CSS assets
    let css_dir = src_dir.join("css");
    fs::create_dir_all(&css_dir)
        .with_context(|| format!("Failed to create {}", css_dir.display()))?;

    fs::write(css_dir.join("slides.css"), BRIDGE_CSS)
        .context("Failed to write slides.css")?;
    fs::write(css_dir.join("slides-print.css"), PRINT_CSS)
        .context("Failed to write slides-print.css")?;

    // Add CSS to [output.html] additional-css
    if doc.get("output").is_none() {
        doc["output"] = Item::Table(Table::new());
    }
    let output = doc["output"].as_table_mut().unwrap();
    if output.get("html").is_none() {
        output.insert("html", Item::Table(Table::new()));
    }
    let html = output["html"].as_table_mut().unwrap();

    let css_files = ["src/css/slides.css", "src/css/slides-print.css"];

    let existing: Vec<String> = html
        .get("additional-css")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let mut arr = html
        .get("additional-css")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    for css in &css_files {
        if !existing.iter().any(|e| e == css) {
            arr.push(*css);
        }
    }

    html.insert("additional-css", Item::Value(Value::Array(arr)));

    fs::write(&toml_path, doc.to_string()).context("Failed to write book.toml")?;

    log::info!("Installed mdbook-slides preprocessor");
    log::info!("  - Updated {}", toml_path.display());
    log::info!("  - Wrote CSS to {}", css_dir.display());

    Ok(())
}
