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

The homepage metadata in `Cargo.toml` currently points to `https://gpm.github.io/geoquery-rs/`. Update `homepage`, `repository`, and `docs/_config.yml` if the GitHub owner or repository name differs.

## Preview this site locally

```sh
cd docs
bundle install
bundle exec jekyll serve
```

Then open `http://127.0.0.1:4000/geoquery-rs/`.
