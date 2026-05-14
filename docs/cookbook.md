---
layout: default
title: Cookbook
description: Practical geoquery-rs recipes for parsing, annotation, typed tables, and publishing checks.
---

## Parse a local series matrix

Use local parsers in tests, pipelines, and reproducible workflows where network access is undesirable.

```rust
fn main() -> geoquery::Result<()> {
    let eset = geoquery::parse_gse_matrix("GSE2553_series_matrix.txt.gz")?;
    println!("{} features x {} samples", eset.nrow(), eset.ncol());
    Ok(())
}
```

## Expand sample characteristics

GEO sample characteristics often mix delimiters or omit keys. `parse_characteristics` recovers key/value pairs when possible and stores unkeyed values as `characteristic_N`.

```rust
let parsed = geoquery::parse_characteristics("disease:control;sex=male | age:72");
assert_eq!(parsed.get("disease").map(String::as_str), Some("control"));
assert_eq!(parsed.get("sex").map(String::as_str), Some("male"));
assert_eq!(parsed.get("age").map(String::as_str), Some("72"));
```

## Join GPL annotations into an ExpressionSet

Join platform metadata without mutating the original expression matrix. Row order, duplicate feature IDs, and missing annotation values are preserved.

```rust
use geoquery::{join_gpl_annotations, parseGEO, parseGPL, GEOObject};

fn main() -> geoquery::Result<()> {
    let parsed = parseGEO("GSE2553_series_matrix.txt.gz", None, None, false, true)?;
    let GEOObject::GSEMatrix(esets) = parsed else {
        return Ok(());
    };

    let gpl = parseGPL("GPL570.annot.gz")?;
    let joined = join_gpl_annotations(&esets[0], &gpl)?;
    println!("feature annotation columns: {}", joined.featureData.ncol());
    Ok(())
}
```

Use a different key when the expression identifiers are gene symbols or Entrez IDs.

```rust
use geoquery::{join_gpl_annotations_by_key, AnnotationKey};
# fn demo(eset: &geoquery::ExpressionSet, gpl: &geoquery::GPL) -> geoquery::Result<()> {
let joined = join_gpl_annotations_by_key(eset, gpl, AnnotationKey::GeneSymbol)?;
println!("{}", joined.featureData.nrow());
# Ok(())
# }
```

## Read typed DataFrame columns

`DataFrame` stores typed column data internally and reconstructs row views only when requested.

```rust
use geoquery::{DataFrame, TypedColumn};

let df = DataFrame::new(
    vec!["probe".into(), "count".into()],
    vec![
        vec![Some("1007_s_at".into()), Some("12".into())],
        vec![Some("117_at".into()), Some("15".into())],
    ],
);

assert_eq!(df.column_names(), &["probe".to_string(), "count".to_string()]);
assert!(matches!(df.typed_column("count"), Some(TypedColumn::Integer(_))));
```

## Build analysis objects

```rust
use geoquery::{gds_to_expression_set, gds_to_ma, parseGDS};

fn main() -> geoquery::Result<()> {
    let gds = parseGDS("GDS507.soft.gz")?;
    let eset = gds_to_expression_set(&gds, false);
    let ma = gds_to_ma(&gds, false);
    println!("ExpressionSet: {} x {}", eset.nrow(), eset.ncol());
    println!("MAList: {} x {}", ma.nrow(), ma.ncol());
    Ok(())
}
```

## Run release checks

Before publishing, run the same checks used by CI.

```sh
cargo fmt --check
cargo test
cargo package --list --allow-dirty
cargo package --allow-dirty --offline
```
