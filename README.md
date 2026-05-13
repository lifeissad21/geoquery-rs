# geoquery-rs

`geoquery-rs` is a native Rust rewrite of the R/Bioconductor
[`GEOquery`](https://github.com/seandavi/GEOquery) package. It reads NCBI Gene
Expression Omnibus (GEO) SOFT files, series matrix files, GEO metadata, GEO
tables, RNA-seq quantification resources, and supplemental file listings without
requiring R or Bioconductor at runtime.

The published package name is `geoquery-rs`; the Rust library name is
`geoquery`.

## Status

This project targets test-suite parity with the reference R package. The Rust
tests in `tests/` mirror the original R `testthat` coverage, with compact local
fixtures under `tests/fixtures/`.

The crate preserves many familiar R names such as `getGEO`, `GDS2MA`,
`GDS2eSet`, `getGEOSuppFiles`, and `searchGEO`, while also exposing idiomatic
snake_case aliases for common entry points.

## Installation

From crates.io:

```sh
cargo add geoquery-rs
```

In Rust code:

```rust
use geoquery::{get_geo, GEOObject, GetGeoOptions};
```

From this repository:

```toml
[dependencies]
geoquery-rs = { path = "." }
```

## Running

Run the included command-line smoke test from this repository:

```sh
cargo run -- GDS507
```

The accession argument is optional. If omitted, the binary fetches `GDS507`:

```sh
cargo run
```

After publishing, install and run the binary with:

```sh
cargo install geoquery-rs
geoquery GDS507
```

Use the crate as a library by adding `geoquery-rs` to `Cargo.toml` and importing
the library as `geoquery`:

```rust
use geoquery::{get_geo, GEOObject, GetGeoOptions};
```

The binary is intentionally small. Real applications should use the library API
so they can choose output formats, cache directories, retry behavior, and how to
handle each `GEOObject` variant.

## Quick Start

Fetch and parse a GEO accession:

```rust,no_run
use geoquery::{get_geo, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let geo = get_geo(GetGeoOptions {
        GEO: Some("GDS507"),
        GSEMatrix: false,
        ..Default::default()
    })?;

    match geo {
        GEOObject::GDS(gds) => {
            println!("accession: {:?}", gds.accession());
            println!("table: {} rows x {} columns", gds.table().nrow(), gds.table().ncol());
        }
        other => println!("received {other:?}"),
    }

    Ok(())
}
```

Parse a local SOFT or series matrix file:

```rust,no_run
use std::path::Path;

fn main() -> geoquery::Result<()> {
    let object = geoquery::parse_geo(
        Path::new("GDS507.soft.gz"),
        None,
        None,
        false,
        true,
    )?;

    println!("{object:?}");
    Ok(())
}
```

Fetch a GSE series matrix, which is the default for GSE accessions:

```rust,no_run
use geoquery::{getGEO, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let geo = getGEO(GetGeoOptions {
        GEO: Some("GSE2553"),
        ..Default::default()
    })?;

    if let GEOObject::GSEMatrix(esets) = geo {
        for eset in esets {
            println!("{} genes x {} samples", eset.nrow(), eset.ncol());
        }
    }

    Ok(())
}
```

Download full GSE SOFT records instead of matrix files:

```rust,no_run
use geoquery::{get_geo, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let geo = get_geo(GetGeoOptions {
        GEO: Some("GSE2553"),
        GSEMatrix: false,
        ..Default::default()
    })?;

    if let GEOObject::GSE(gse) = geo {
        println!("{} GSM records", gse.gsm_list().len());
        println!("{} GPL records", gse.gpl_list().len());
    }

    Ok(())
}
```

## R GEOquery to Rust Mapping

| R GEOquery | geoquery-rs |
| --- | --- |
| `getGEO("GDS507")` | `get_geo(GetGeoOptions { GEO: Some("GDS507"), ..Default::default() })` |
| `getGEO("GSE2553", GSEMatrix = FALSE)` | `get_geo(GetGeoOptions { GEO: Some("GSE2553"), GSEMatrix: false, ..Default::default() })` |
| `parseGEO("file.soft.gz")` | `parse_geo("file.soft.gz", None, None, false, true)` |
| `Meta(x)` | `x.meta()` or the compatibility trait `x.Meta()` |
| `Table(x)` | `x.table()` or the compatibility trait `x.Table()` |
| `Columns(x)` | `x.columns()` or the compatibility trait `x.Columns()` |
| `GSMList(gse)` | `gse.gsm_list()` or `gse.GSMList()` |
| `GPLList(gse)` | `gse.gpl_list()` or `gse.GPLList()` |
| `GDS2MA(gds)` | `gds_to_ma(&gds, do_log2)` or `GDS2MA(&gds, do_log2)` |
| `GDS2eSet(gds)` | `gds_to_expression_set(&gds, do_log2)` or `GDS2eSet(&gds, do_log2)` |
| `getGEOSuppFiles("GSE...")` | `getGEOSuppFiles("GSE...", make_directory, base_dir, fetch_files, filter_regex)` |
| `searchGEO("...")` | `searchGEO("...", retmax)` |

## Data Model

The rewrite replaces R S4 and Bioconductor objects with Rust-native structs:

| Bioconductor/R object | Rust replacement |
| --- | --- |
| `GDS`, `GPL`, `GSM`, `GSE` | `GDS`, `GPL`, `GSM`, `GSE` structs |
| `GEODataTable` | `GEODataTable` with `columns` and `table` data frames |
| `data.frame` | `DataFrame` with string-backed rows and typed column views |
| `ExpressionSet` | `ExpressionSet` with `NumericMatrix`, phenotype data, feature data, annotation, and experiment metadata |
| `MAList` | `MAList` with matrix data, targets, genes, and notes |
| `SummarizedExperiment` | `SummarizedExperiment` with named assays, row data, column data, metadata, and genome info |

`DataFrame::typed_column` can derive integer or floating-point views from
string-backed GEO columns when the values are parseable.

## Supplemental Files

List supplemental files without downloading:

```rust,no_run
use std::path::Path;

fn main() -> geoquery::Result<()> {
    let files = geoquery::getGEOSuppFiles(
        "GSE161228",
        true,
        Path::new("geo-downloads"),
        false,
        Some(r"\.txt\.gz$"),
    )?;

    for file in files {
        println!("{}\t{}", file.fname, file.url);
    }

    Ok(())
}
```

Set `fetch_files` to `true` to download matching files.

## Search

`searchGEO` and `searchFieldsGEO` use NCBI E-utilities and require network
access:

```rust,no_run
fn main() -> geoquery::Result<()> {
    let result = geoquery::searchGEO("breast cancer AND Homo sapiens[Organism]", 20)?;
    println!("{} rows", result.nrow());
    Ok(())
}
```

## CLI Smoke Test

The crate also includes a small `geoquery` binary:

```sh
cargo run -- GDS507
```

It fetches one accession, prints selected metadata, and shows the first table
rows.

## Development

Run local fixture tests:

```sh
cargo test
```

Run live NCBI/GEO tests:

```sh
cargo test -- --ignored
```

Build Rust API documentation:

```sh
cargo doc --no-deps --open
```

Build docs with warnings treated as errors:

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

The generated local API docs are written to:

```text
target/doc/geoquery/index.html
```

After publishing to crates.io, docs.rs will build and host the public API docs
at:

```text
https://docs.rs/geoquery-rs
```

Package the crate locally:

```sh
cargo package --allow-dirty
```

Publish to crates.io:

```sh
cargo login
cargo publish --dry-run
cargo publish
```

The project homepage is a Jekyll site in `docs/`. To publish it, push the
repository to GitHub and configure Pages to deploy from the `docs/` folder on
the default branch. The Cargo homepage metadata currently points to:

```text
https://gpm.github.io/geoquery-rs/
```

Change `homepage` in `Cargo.toml` if the GitHub owner or repository name is
different.

Preview the Jekyll site locally:

```sh
cd docs
bundle install
bundle exec jekyll serve
```

The `target/` build output and generated crate archives are excluded from
publication.

## License

MIT. See `LICENSE`.
