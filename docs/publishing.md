---
layout: default
title: Publishing
description: Build docs, publish to crates.io, and deploy the GitHub Pages site.
---

## Build local API docs

```sh
cargo doc --no-deps --open
```

The local rustdoc entry point is:

```text
target/doc/geoquery/index.html
```

For release checks, build with warnings treated as errors:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Run release checks

```sh
cargo fmt --check
cargo test
cargo package --list --allow-dirty --offline
cargo package --allow-dirty --offline
```

The GitHub Actions `Cargo checks` workflow runs `cargo fmt --check` and
`cargo test` on pushes, pull requests, and manual dispatches.

Run live GEO tests when network access is available:

```sh
cargo test -- --ignored
```

## Publish to crates.io

```sh
cargo login
cargo publish --dry-run
cargo publish
```

After publishing, docs.rs will automatically build API documentation at [https://docs.rs/geoquery-rs](https://docs.rs/geoquery-rs).

## Publish GitHub Pages

This Jekyll site lives in `docs/`.

1. Push the repository to GitHub.
2. Open `Settings > Pages`.
3. Choose `Deploy from a branch`.
4. Select the default branch and the `/docs` folder.
5. Save the settings.

The homepage metadata in `Cargo.toml` currently points to `https://lifeissad21.github.io/geoquery-rs/`. Update `homepage`, `repository`, and `docs/_config.yml` if the GitHub owner or repository name differs.

The site uses `permalink: pretty` in `docs/_config.yml` so routes such as
`/geoquery-rs/examples/` build as `examples/index.html` in the Pages artifact.

## Preview this site locally

```sh
cd docs
bundle install
bundle exec jekyll serve
```

Then open `http://127.0.0.1:4000/geoquery-rs/`.
