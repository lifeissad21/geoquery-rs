# AI Artifact And Changes

This file consolidates the project notes that were previously split across
`BIOCONDUCTOR_OBJECTS_TODO.md`, `PUBLISHING.md`, `REWRITE_NOTES.md`,
`TEST_PARITY.md`, and `CHANGELOG.md`.

It records what the AI-assisted rewrite changed, the current Bioconductor-style
object replacement status, test parity, and publishing checklist.

## Native GEO/Bioconductor Expansion

This update expands functionality beyond current GEOquery parity while keeping
existing public workflows and tests intact.

### Added

- Added `AnnotationKey` with probe ID, gene symbol, Entrez, and custom-column
  join modes.
- Added `join_gpl_annotations` and `join_gpl_annotations_by_key` to join GPL
  platform tables into `ExpressionSet.featureData` / `feature_data`.
- Added resilient `parse_characteristics` handling for repeated, collapsed,
  mixed-delimiter, malformed, and unkeyed GEO sample characteristics.
- Added schema-aware `ColumnData` and `Value` storage on `DataFrame`.
- Removed public `DataFrame::rows` and `DataFrame::columns` storage fields in
  favor of typed column storage and explicit accessors.
- Added expanded GEO-specific error variants for annotation, characteristics,
  matrix, SOFT, RNA-seq, network, and parse failures.
- Added snake-case helpers `parse_gse`, `parse_gse_matrix`, and
  `get_rna_seq_data`.
- Added Rustdoc examples for GPL annotation joins, GDS conversion helpers,
  GSE parsing, GSE matrix parsing, and RNA-seq access.

### Fixtures And Tests

- Added local malformed-characteristic GSE matrix fixtures, including a
  compressed `.txt.gz` version.
- Added local malformed GSM, missing-header SOFT, truncated SOFT, and duplicate
  GPL probe fixtures.
- Added network-free tests for duplicate probes, missing annotation IDs,
  partial annotation matches, large generated GPL tables, malformed GPL tables,
  robust characteristic parsing, compressed matrix parsing, and typed data frame
  inference.

### Compatibility Notes

- `DataFrame` no longer stores public `rows` / `columns` vectors. Typed
  `DataFrame::data` is the storage source of truth, with `column_names`,
  `row_values`, `column_values`, and `get_by_index` for access.
- Deprecated compatibility helpers `DataFrame::rows()` and
  `DataFrame::columns()` reconstruct the old row/column views for migration.
- GPL annotation joins clone the input `ExpressionSet`; expression matrices are
  never mutated.
- Existing parity tests continue to pass.

## GEOquery Rust Rewrite Notes

### R Package Dependencies

The reference R package declares these runtime dependencies:

- `Depends`: `methods`, `Biobase`
- `Imports`: `readr`, `xml2`, `dplyr`, `data.table`, `tidyr`, `magrittr`, `limma`, `curl`, `rentrez`, `R.utils`, `stringr`, `SummarizedExperiment`, `S4Vectors`, `rvest`, `httr2`
- `Suggests`: `knitr`, `rmarkdown`, `BiocGenerics`, `testthat`, `covr`, `markdown`, `quarto`, `DropletUtils`, `SingleCellExperiment`

The Rust crate replaces the R/Bioconductor classes with Rust-native structs:
`GDS`, `GPL`, `GSM`, `GSE`, `GEODataTable`, `ExpressionSet`, and `MAList`.

### Test Porting

The tests in `tests/` mirror the R `testthat` files. Local fixture tests run by
default. Tests that require live NCBI network access are present and marked with
`#[ignore]`, so they can be run explicitly with:

```sh
cargo test -- --ignored
```

## Bioconductor Object Replacement Status

This section tracks the R/Bioconductor-heavy objects from the reference
GEOquery package and the current Rust-native replacement status.

The current Rust crate has test-suite parity with the reference R tests:

```sh
cargo test
cargo test -- --ignored
```

Both passed in the latest verification. The `--ignored` suite requires live
NCBI/GEO network access.

### Implemented

| R/Bioconductor object | Rust replacement | Status | Notes |
| --- | --- | --- | --- |
| `ExpressionSet` | `ExpressionSet` | Implemented for tested GEOquery behavior | Stores a numeric expression matrix, feature names, sample names, phenotype data, feature data, annotation, raw experiment headers, and typed experiment metadata. |
| `SummarizedExperiment` | `SummarizedExperiment` | Implemented for RNA-seq tests | `getRNASeqData` now returns a native Rust `SummarizedExperiment` with a `counts` assay, `rowData`, `colData`, metadata, genome info, and creation timestamp. |
| `AnnotatedDataFrame` | `AnnotatedDataFrame` | Implemented | Wraps a `DataFrame` plus variable metadata. Used for phenotype and feature metadata equivalents. |
| `MAList` | `MAList` | Implemented for compatibility | Keeps the familiar limma-like structure with `M`, optional `A`, targets, optional genes, notes, row names, and column names. |
| `MIAME` | `ExperimentData` | Implemented for tested metadata | Captures contact name, lab, contact email, title, URL, PubMed IDs, abstract text, and the full raw header map. |
| `S4Vectors::metadata` | explicit metadata fields | Implemented | `SummarizedExperiment` stores metadata directly, including genome information and creation timestamp. |
| `assays` | `BTreeMap<String, NumericMatrix>` | Implemented | RNA-seq counts are stored under the `counts` assay name. |
| `rowData` | `DataFrame` | Implemented | Used for gene annotation in `SummarizedExperiment`. |
| `colData` | `DataFrame` | Implemented | Used for sample metadata in `SummarizedExperiment`; currently minimal for RNA-seq count-only construction. |
| row names | `DataFrame::row_names`, `NumericMatrix::row_names` | Implemented | Used by conversions, GSE matrix parsing, and RNA-seq count matrices. |
| column names | `DataFrame::column_names`, `NumericMatrix::column_names` | Implemented | Used by parsed tables, expression matrices, and assays. |
| variable metadata | `DataFrame::column_metadata` and `AnnotatedDataFrame::var_metadata` | Implemented for current parsers | Parsed GEO column descriptions now populate `DataFrame::column_metadata`, and `GDS2eSet` builds `AnnotatedDataFrame::var_metadata` from available descriptions. |
| typed column storage | `DataFrame::data`, `ColumnData`, `Value`, `DataFrame::typed_column`, and `TypedColumn` | Implemented | Stores data as typed columns and reconstructs row-style views only through explicit compatibility helpers. |
| typed matrix helpers | `NumericMatrix::get`, `row_name`, `column_name` | Implemented | Gives ergonomic typed access to expression and assay matrices. |
| idiomatic Rust API aliases | snake_case wrappers | Implemented for public convenience/core helpers | Added aliases for URL, browse, supplemental URL, `getGEO`, `parseGEO`, `getGEOfile`, `GDS2MA`, and `GDS2eSet`, plus snake_case accessors on GEO record structs. |
| domain error variants | `GeoError` variants | Implemented and expanded | Includes structured variants for accession, matrix, supplemental, annotation, GPL joins, SOFT/GSE matrix/RNA-seq structure, parser, and network failures. |
| local compressed fixtures | Rust integration tests | Implemented for core parser paths | Added non-network compressed fixtures for GSM, GPL, malformed GSE matrix characteristic parsing, and RNA-seq counts/annotation parsing. |
| Rustdoc examples | public API docs | Implemented for core workflows | Added doctested examples and crate-level cookbook recipes for parsing, annotation joins, typed data frames, and conversions. |

### GEOquery S4 Class Replacements

| R object | Rust replacement | Status | Notes |
| --- | --- | --- | --- |
| `GEODataTable` | `GEODataTable` | Implemented | Holds column descriptions and parsed table data. |
| `GEOData` | `GEOData` | Implemented as compatibility shape | Base-like struct containing headers and a data table. |
| `GDS` | `GDS` | Implemented | Supports parsing, metadata, table access, and GDS conversions. GPL attachment is represented as optional. |
| `GPL` | `GPL` | Implemented | Supports live GPL tests including quoted, short, and no-table GPL records. |
| `GSM` | `GSM` | Implemented | Supports live GSM parsing and gzip-content detection. |
| `GSE` | `GSE` | Implemented | Supports full SOFT parsing for `GSEMatrix = false`, plus default matrix parsing through `GSEMatrix`. |

### Implemented Functions Affected By These Objects

| Function | Status | Notes |
| --- | --- | --- |
| `getGEO(..., GSEMatrix = TRUE)` | Implemented for tested cases | Downloads and parses GSE series matrix files into `GSEMatrix(Vec<ExpressionSet>)`. |
| `getGEO(..., GSEMatrix = FALSE)` | Implemented for tested cases | Downloads and parses full SOFT GSE into `GSE`. |
| `parseGSEMatrix` | Implemented for tested cases | Preserves expression shape, feature/sample names, phenotype data, feature data row names, annotation, and experiment metadata. |
| `GDS2eSet` | Implemented for tested cases | Produces native `ExpressionSet` with expression matrix, feature rows, phenotype data, annotation, and experiment metadata. |
| `GDS2MA` | Implemented for tested cases | Produces native `MAList` with matrix dimensions, row names, column names, targets, and notes. |
| `getRNASeqData` | Implemented for tested cases | Produces native `SummarizedExperiment` with `counts`, annotation, sample metadata shell, genome metadata, and timestamp. |

### Still Needs Work

These are not blocking the current ported test suite, but they matter for a
more complete replacement of the original Bioconductor behavior.

| Area | What remains |
| --- | --- |
| Full Bioconductor semantic parity | The Rust structs match GEOquery test expectations, but they do not replicate every method, validation rule, coercion, or edge behavior from Bioconductor classes. |
| Rich `AnnotatedDataFrame` metadata | Implemented for the current native data model and tested conversion paths. Future work only applies if deeper Bioconductor-specific metadata semantics are required. |
| Error taxonomy | Structured domain variants exist for known GEO-specific failures. Some low-level network and parser errors intentionally remain wrapped to preserve source context. |
| API naming cleanup | Core function aliases and struct accessor aliases now exist. R-compatible trait names remain for familiarity and backward compatibility with the port. |
| Documentation examples | Rustdoc and Jekyll docs include cookbook-style examples. More examples can be added as new workflows are implemented. |
| Broader fixture coverage | Local compressed GSM, GPL, GSE matrix, and RNA-seq fixtures now cover core parser paths. Supplemental listing fixtures would require an HTTP mocking layer and are not needed for current parity. |
| Additional GPL edge cases | Optional GPL annotation joins are implemented for probe, gene symbol, Entrez, and custom keys. Future work can add organism-specific normalization rules if needed. |
| Additional characteristic edge cases | Implemented and locally tested for repeated, collapsed, mixed-delimiter, and unkeyed GSE matrix characteristics. Future work can add more real-world malformed fixtures as they are found. |

### Current Parity Claim

The current claim is **test-suite parity with the reference R tests**. All R
`testthat` files have Rust counterparts, and both the local and live ignored
Rust suites passed in the latest run.

The remaining rows above are intentionally framed as future expansion areas,
not blockers for parity with the current GEOquery tests. Exact method-for-method
Bioconductor compatibility is not a native Rust target because this crate does
not embed R or Bioconductor.

## GEOquery Test Parity

The Rust test suite now has coverage corresponding to every R `testthat` file
from the reference GEOquery package.

### Commands

Run local fixture/unit tests:

```sh
cargo test
```

Run live NCBI/GEO parity tests:

```sh
cargo test -- --ignored
```

Run everything:

```sh
cargo test -- --include-ignored
```

### R Test Files Represented

| R test file | Rust test file | Status |
| --- | --- | --- |
| `test_GDS.R` | `tests/test_gds.rs` | Ported |
| `test_GEO_conversions.R` | `tests/test_geo_conversions.rs` | Ported |
| `test_GPL.R` | `tests/test_gpl.rs` | Ported; live tests ignored by default |
| `test_GSE.R` | `tests/test_gse.rs` | Ported; live tests ignored by default |
| `test_GSM.R` | `tests/test_gsm.rs` | Ported; live test ignored by default |
| `test_fetch_GPL_false.R` | `tests/test_fetch_gpl_false.rs` | Ported; live tests ignored by default |
| `test_geo_browse.R` | `tests/test_geo_browse.rs` | Ported |
| `test_geo_rnaseq.R` | `tests/test_geo_rnaseq.rs` | Ported; live tests ignored by default |
| `test_search.R` | `tests/test_search.rs` | Ported; live test ignored by default |
| `test_supp_files.R` | `tests/test_supp_files.rs` | Ported; live/download tests ignored by default |

### Latest Verification

Both commands were run successfully after the latest Rust implementation work:

```sh
cargo test
cargo test -- --ignored
```

The ignored suite requires network access to NCBI and downloads GEO files.

## Publishing Checklist

Project name: `geoquery-rs`

Crate package name: `geoquery-rs`

Rust library import name: `geoquery`

Repository: `https://github.com/lifeissad21/geoquery-rs`

GitHub Pages homepage: `https://lifeissad21.github.io/geoquery-rs/`

Primary run commands:

```sh
cargo run -- GDS507
cargo install geoquery-rs
geoquery GDS507
```

### Before Publishing

1. Confirm the crates.io name is available:

   ```sh
   cargo search geoquery-rs --limit 5
   ```

2. Confirm the `repository` and `homepage` values in `Cargo.toml` match the
   GitHub owner and repository name. They are currently set to:

   ```text
   https://github.com/lifeissad21/geoquery-rs
   https://lifeissad21.github.io/geoquery-rs/
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

### Build Documentation

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

### Publish GitHub Pages

The Jekyll homepage and documentation site is in `docs/`.

1. Push this repository to GitHub.
2. In the GitHub repository, open `Settings > Pages`.
3. Set source to `Deploy from a branch`.
4. Select the default branch and `/docs` folder.
5. Save. GitHub will publish the site at the `homepage` URL in `Cargo.toml`.

If the GitHub owner is not `lifeissad21`, update `repository` and `homepage` in
`Cargo.toml`, plus `url`, `baseurl`, `repository`, and `crate.github_url` in
`docs/_config.yml`, before publishing the crate.

### Continuous Integration

The `Cargo checks` GitHub Actions workflow runs on pushes, pull requests, and
manual dispatches. It enforces:

```sh
cargo fmt --check
cargo test
```

### Notes

- `GEOquery/` may exist locally as a development reference checkout, but it is
  ignored by git and excluded from the published crate. Keep publishable tests
  on the minimal fixtures under `tests/fixtures/`.
- `target/` and generated `.crate` archives are ignored and excluded.
- The project is MIT licensed, matching the local reference package license.
