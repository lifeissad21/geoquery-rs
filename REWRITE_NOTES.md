# GEOquery Rust Rewrite Notes

## R Package Dependencies

The reference R package declares these runtime dependencies:

- `Depends`: `methods`, `Biobase`
- `Imports`: `readr`, `xml2`, `dplyr`, `data.table`, `tidyr`, `magrittr`, `limma`, `curl`, `rentrez`, `R.utils`, `stringr`, `SummarizedExperiment`, `S4Vectors`, `rvest`, `httr2`
- `Suggests`: `knitr`, `rmarkdown`, `BiocGenerics`, `testthat`, `covr`, `markdown`, `quarto`, `DropletUtils`, `SingleCellExperiment`

The Rust crate replaces the R/Bioconductor classes with Rust-native structs:
`GDS`, `GPL`, `GSM`, `GSE`, `GEODataTable`, `ExpressionSet`, and `MAList`.

## Test Porting

The tests in `tests/` mirror the R `testthat` files. Local fixture tests run by
default. Tests that require live NCBI network access are present and marked with
`#[ignore]`, so they can be run explicitly with:

```sh
cargo test -- --ignored
```
