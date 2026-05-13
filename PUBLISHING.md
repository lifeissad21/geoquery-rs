# Publishing Checklist

Project name: `geoquery-rs`

Crate package name: `geoquery-rs`

Rust library import name: `geoquery`

Repository: `https://github.com/gpm/geoquery-rs`

GitHub Pages homepage: `https://gpm.github.io/geoquery-rs/`

Primary run commands:

```sh
cargo run -- GDS507
cargo install geoquery-rs
geoquery GDS507
```

## Before Publishing

1. Confirm the crates.io name is available:

   ```sh
   cargo search geoquery-rs --limit 5
   ```

2. Confirm the `repository` and `homepage` values in `Cargo.toml` match the
   GitHub owner and repository name. They are currently set to:

   ```text
   https://github.com/gpm/geoquery-rs
   https://gpm.github.io/geoquery-rs/
   ```

   `documentation` points at docs.rs.

3. Run formatting and tests:

   ```sh
   cargo fmt --check
   cargo test
   ```

4. Run live network tests when NCBI/GEO access is available:

   ```sh
   cargo test -- --ignored
   ```

5. Inspect the crate contents:

   ```sh
   cargo package --list --allow-dirty
   ```

6. Build the package:

   ```sh
   cargo package --allow-dirty --offline
   ```

7. Run the online publish dry run:

   ```sh
   cargo publish --dry-run
   ```

8. Publish:

   ```sh
   cargo publish
   ```

## Build Documentation

Build local API docs:

```sh
cargo doc --no-deps --open
```

Build API docs with rustdoc warnings treated as errors:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The local entry point is:

```text
target/doc/geoquery/index.html
```

After `cargo publish`, docs.rs builds the API docs automatically at:

```text
https://docs.rs/geoquery-rs
```

## Publish GitHub Pages

The Jekyll homepage and documentation site is in `docs/`.

1. Push this repository to GitHub.
2. In the GitHub repository, open `Settings > Pages`.
3. Set source to `Deploy from a branch`.
4. Select the default branch and `/docs` folder.
5. Save. GitHub will publish the site at the `homepage` URL in `Cargo.toml`.

If the GitHub owner is not `gpm`, update `repository` and `homepage` in
`Cargo.toml`, plus `url`, `baseurl`, `repository`, and `crate.github_url` in
`docs/_config.yml`, before publishing the crate.

## Notes

- `GEOquery/` may exist locally as a development reference checkout, but it is
  ignored by git and excluded from the published crate. Keep publishable tests
  on the minimal fixtures under `tests/fixtures/`.
- `target/` and generated `.crate` archives are ignored and excluded.
- The project is MIT licensed, matching the local reference package license.
