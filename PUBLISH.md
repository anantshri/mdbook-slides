# Publishing to crates.io

This document covers the release process for `mdbook-slides`.

## One-time setup

1. Create a crates.io account at <https://crates.io/> and verify your email (required by crates.io before publishing).
2. Generate an API token at <https://crates.io/me> with the `publish-new` and `publish-update` scopes.
3. Authenticate locally:
   ```sh
   cargo login
   ```
   Paste the token when prompted. It is stored in `~/.cargo/credentials.toml`.

## Pre-flight checklist

Before publishing a new version:

- [ ] Working tree is clean: `git status` shows no uncommitted changes.
- [ ] You are on `main` and up to date: `git pull --ff-only`.
- [ ] `version` in `Cargo.toml` is bumped (follow [SemVer](https://semver.org/) — breaking API changes require a major bump; for `0.x` releases a minor bump signals breaking).
- [ ] `CHANGELOG.md` has a section for the new version with the release date.
- [ ] Tests pass: `cargo test`.
- [ ] No warnings: `cargo build --release` and `cargo clippy --all-targets -- -D warnings` (if you use clippy).
- [ ] The `test-book/` builds end-to-end with the latest local mdbook:
   ```sh
   cargo install --path . --force
   cd test-book && mdbook build
   ```

## Dry run

Always do a packaging dry-run before publishing for real:

```sh
cargo publish --dry-run
```

This packages the crate, runs all checks crates.io will run, and verifies it compiles in isolation — without uploading anything. Inspect the file list it prints to confirm no unintended files are included (the `[package]` section's `include`/`exclude` controls this if needed).

To inspect the exact tarball that will be uploaded:

```sh
cargo package --list
```

## Publish

```sh
cargo publish
```

The crate appears on crates.io within a few seconds; docs.rs builds the documentation automatically (usually within a few minutes).

## Tag the release

```sh
git tag -a v0.3.0 -m "Release 0.3.0"
git push origin v0.3.0
```

Use the `vX.Y.Z` form to match the existing `v0.1.1` tag.

## Post-publish

- Verify the crate page at <https://crates.io/crates/mdbook-slides> shows the new version.
- Verify docs at <https://docs.rs/mdbook-slides>.
- (Optional) Draft a GitHub release pointing at the new tag, with the changelog excerpt as the body.

## Yanking

If you discover a critical issue after publishing, yank the bad version. Yanking prevents new projects from depending on it but does **not** remove it (existing lockfiles continue to work):

```sh
cargo yank --version 0.3.0
```

To un-yank:

```sh
cargo yank --version 0.3.0 --undo
```

Published versions cannot be deleted or overwritten — only yanked. Always bump to a new version to fix issues.
