use anyhow::Result;
use clap::{Arg, Command};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use mdbook_slides::install;
use mdbook_slides::SlidesPreprocessor;
use std::io;
use std::path::PathBuf;
use std::process;

fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("mdbook-slides")
        .about("An mdbook preprocessor for slide presentations")
        .subcommand(
            Command::new("supports")
                .about("Check if a renderer is supported")
                .arg(Arg::new("renderer").required(true)),
        )
        .subcommand(
            Command::new("install")
                .about("Install the preprocessor into an mdbook project")
                .arg(
                    Arg::new("dir")
                        .help("Root directory of the mdbook project (default: current directory)")
                        .default_value("."),
                ),
        )
        .get_matches();

    let preprocessor = SlidesPreprocessor;

    match matches.subcommand() {
        Some(("supports", sub_m)) => {
            let renderer = sub_m.get_one::<String>("renderer").unwrap();
            if preprocessor.supports_renderer(renderer) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        Some(("install", sub_m)) => {
            let dir = sub_m.get_one::<String>("dir").unwrap();
            install::install(&PathBuf::from(dir))?;
        }
        _ => {
            // Standard preprocessor protocol: read from stdin, write to stdout
            let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

            if ctx.mdbook_version != mdbook::MDBOOK_VERSION {
                log::warn!(
                    "mdbook version mismatch: expected {}, got {}",
                    mdbook::MDBOOK_VERSION,
                    ctx.mdbook_version
                );
            }

            let processed = preprocessor.run(&ctx, book)?;
            serde_json::to_writer(io::stdout(), &processed)?;
        }
    }

    Ok(())
}
