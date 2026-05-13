---
layout: default
title: Getting Started
description: Install, run, and use geoquery-rs as a Rust library or CLI smoke test.
---

The published package is `geoquery-rs`. The Rust library is imported as `geoquery`, which keeps downstream code concise and compatible with the original project name.

## Install

```sh
cargo add geoquery-rs
```

Or, from a local checkout:

```toml
[dependencies]
geoquery-rs = { path = "." }
```

## Run the CLI smoke test

```sh
cargo run -- GDS507
```

The CLI fetches a single GEO accession, prints selected metadata, and previews table dimensions. If no accession is supplied, it defaults to `GDS507`.

```sh
cargo run
```

After publishing, install the binary with Cargo:

```sh
cargo install geoquery-rs
geoquery GDS507
```

## Use as a library

```rust
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

## Network behavior

High-level download functions use NCBI GEO and NCBI E-utilities. Local parser functions such as `parseGDS`, `parseGSE`, and `parseGSEMatrix` do not need network access.

## Development tests

```sh
cargo test
cargo test -- --ignored
```

The ignored tests require live NCBI/GEO network access and may download GEO files.
