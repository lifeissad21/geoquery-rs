---
layout: default
title: Examples
description: Common geoquery-rs workflows with runnable Rust snippets.
---

## Fetch a GDS dataset

```rust
use geoquery::{get_geo, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let object = get_geo(GetGeoOptions {
        GEO: Some("GDS507"),
        GSEMatrix: false,
        ..Default::default()
    })?;

    if let GEOObject::GDS(gds) = object {
        println!("metadata keys: {}", gds.meta().len());
        println!("table shape: {} x {}", gds.table().nrow(), gds.table().ncol());
    }

    Ok(())
}
```

## Parse a local file

```rust
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

## Fetch GSE matrix files

```rust
use geoquery::{getGEO, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let object = getGEO(GetGeoOptions {
        GEO: Some("GSE2553"),
        ..Default::default()
    })?;

    if let GEOObject::GSEMatrix(esets) = object {
        for eset in esets {
            println!("{} features x {} samples", eset.nrow(), eset.ncol());
        }
    }

    Ok(())
}
```

## Fetch full GSE SOFT records

```rust
use geoquery::{get_geo, GEOObject, GetGeoOptions};

fn main() -> geoquery::Result<()> {
    let object = get_geo(GetGeoOptions {
        GEO: Some("GSE2553"),
        GSEMatrix: false,
        ..Default::default()
    })?;

    if let GEOObject::GSE(gse) = object {
        println!("samples: {}", gse.gsm_list().len());
        println!("platforms: {}", gse.gpl_list().len());
    }

    Ok(())
}
```

## Convert GDS to analysis objects

```rust
use geoquery::{gds_to_expression_set, gds_to_ma, parseGDS};

fn main() -> geoquery::Result<()> {
    let gds = parseGDS("GDS507.soft.gz")?;
    let eset = gds_to_expression_set(&gds, false);
    let malist = gds_to_ma(&gds, false);

    println!("ExpressionSet: {} x {}", eset.nrow(), eset.ncol());
    println!("MAList: {} x {}", malist.nrow(), malist.ncol());

    Ok(())
}
```

## List supplemental files

```rust
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

## Read RNA-seq quantification resources

```rust
fn main() -> geoquery::Result<()> {
    if geoquery::hasRNASeqQuantifications("GSE164073")? {
        let se = geoquery::getRNASeqData("GSE164073")?;
        let counts = se.assay("counts").expect("counts assay");
        println!("counts: {} x {}", counts.nrow(), counts.ncol());
    }

    Ok(())
}
```

## Join GPL platform annotations

```rust
use geoquery::{join_gpl_annotations, parse_gse_matrix, parseGPL};

fn main() -> geoquery::Result<()> {
    let eset = parse_gse_matrix("GSE2553_series_matrix.txt.gz")?;
    let gpl = parseGPL("GPL570.annot.gz")?;
    let annotated = join_gpl_annotations(&eset, &gpl)?;

    println!("feature annotations: {}", annotated.featureData.ncol());
    Ok(())
}
```

## Read typed table columns

```rust
use geoquery::{DataFrame, TypedColumn};

fn main() {
    let df = DataFrame::new(
        vec!["probe".into(), "count".into()],
        vec![
            vec![Some("1007_s_at".into()), Some("12".into())],
            vec![Some("117_at".into()), Some("15".into())],
        ],
    );

    if let Some(TypedColumn::Integer(counts)) = df.typed_column("count") {
        println!("first count: {:?}", counts[0]);
    }
}
```

## Search GEO

```rust
fn main() -> geoquery::Result<()> {
    let result = geoquery::searchGEO("breast cancer AND Homo sapiens[Organism]", 20)?;
    println!("search rows: {}", result.nrow());
    Ok(())
}
```
