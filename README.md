# geoquery-rs

`geoquery-rs` is a native Rust rewrite of the R/Bioconductor
[`GEOquery`](https://github.com/seandavi/GEOquery) package. It reads NCBI Gene
Expression Omnibus (GEO) SOFT files, series matrix files, metadata tables,
RNA-seq quantification resources, and supplemental file listings without
requiring R or Bioconductor at runtime.

The crates.io package is `geoquery-rs`; the Rust library is imported as
`geoquery`.

## Highlights

- GEO records: `GDS`, `GPL`, `GSM`, `GSE`
- Analysis objects: `ExpressionSet`, `SummarizedExperiment`,
  `AnnotatedDataFrame`, `MAList`
- Parsing and download helpers: `get_geo`, `getGEO`, `parse_geo`,
  `parseGSEMatrix`, `getGEOSuppFiles`, `searchGEO`
- Native Rust table storage with typed `DataFrame` columns
- Optional GPL annotation joins and robust sample characteristic parsing
- Ported Rust test coverage corresponding to the reference GEOquery R tests

## Install

```sh
cargo add geoquery-rs
```

From a local checkout:

```toml
[dependencies]
geoquery-rs = { path = "." }
```

## Quick Start

```rust,no_run
use geoquery::{get_geo, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let geo = get_geo(GetGeoOptions {
        GEO: Some("GDS507"),
        GSEMatrix: false,
        ..Default::default()
    })?;

    if let GEOObject::GDS(gds) = geo {
        println!("{} rows x {} columns", gds.table().nrow(), gds.table().ncol());
    }

    Ok(())
}
```

Run the included CLI smoke test:

```sh
cargo run -- GDS507
```

## Documentation

- [Getting started](docs/getting-started.md)
- [API reference overview](docs/api.md)
- [Cookbook recipes](docs/cookbook.md)
- [R GEOquery mapping](docs/r-mapping.md)
- [AI artifact and changes](CHANGES.md)

Generated Rust API documentation is published on
[docs.rs](https://docs.rs/geoquery-rs) after release.

## Development

```sh
cargo fmt --check
cargo test
```

Live GEO/NCBI tests are ignored by default:

```sh
cargo test -- --ignored
```

## License

MIT. See [LICENSE](LICENSE).
