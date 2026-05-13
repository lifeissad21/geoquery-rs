# Bioconductor Object Replacement Status

This file tracks the R/Bioconductor-heavy objects from the reference
GEOquery package and the current Rust-native replacement status.

The current Rust crate has test-suite parity with the reference R tests:

```sh
cargo test
cargo test -- --ignored
```

Both passed in the latest verification. The `--ignored` suite requires live
NCBI/GEO network access.

## Implemented

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
| column names | `DataFrame::columns`, `NumericMatrix::column_names` | Implemented | Used by parsed tables, expression matrices, and assays. |
| variable metadata | `DataFrame::column_metadata` and `AnnotatedDataFrame::var_metadata` | Implemented for current parsers | Parsed GEO column descriptions now populate `DataFrame::column_metadata`, and `GDS2eSet` builds `AnnotatedDataFrame::var_metadata` from available descriptions. |
| typed column helpers | `DataFrame::typed_column` and `TypedColumn` | Implemented as typed views | Provides integer, float, and text views over string-backed data without changing storage layout. |
| typed matrix helpers | `NumericMatrix::get`, `row_name`, `column_name` | Implemented | Gives ergonomic typed access to expression and assay matrices. |
| idiomatic Rust API aliases | snake_case wrappers | Implemented for public convenience/core helpers | Added aliases for URL, browse, supplemental URL, `getGEO`, `parseGEO`, `getGEOfile`, `GDS2MA`, and `GDS2eSet`, plus snake_case accessors on GEO record structs. |
| domain error variants | `GeoError` variants | Partially implemented | Added structured variants for invalid accessions, entity mismatch, missing matrix files, missing supplemental files, and missing fields. |
| local compressed fixtures | Rust integration tests | Implemented for core parser paths | Added non-network compressed fixtures for GSM, GPL, GSE matrix characteristic parsing, and RNA-seq counts/annotation parsing. |
| Rustdoc examples | public API docs | Partially implemented | Added doctested examples for common URL/browse/supplemental helpers. |

## GEOquery S4 Class Replacements

| R object | Rust replacement | Status | Notes |
| --- | --- | --- | --- |
| `GEODataTable` | `GEODataTable` | Implemented | Holds column descriptions and parsed table data. |
| `GEOData` | `GEOData` | Implemented as compatibility shape | Base-like struct containing headers and a data table. |
| `GDS` | `GDS` | Implemented | Supports parsing, metadata, table access, and GDS conversions. GPL attachment is represented as optional. |
| `GPL` | `GPL` | Implemented | Supports live GPL tests including quoted, short, and no-table GPL records. |
| `GSM` | `GSM` | Implemented | Supports live GSM parsing and gzip-content detection. |
| `GSE` | `GSE` | Implemented | Supports full SOFT parsing for `GSEMatrix = false`, plus default matrix parsing through `GSEMatrix`. |

## Implemented Functions Affected By These Objects

| Function | Status | Notes |
| --- | --- | --- |
| `getGEO(..., GSEMatrix = TRUE)` | Implemented for tested cases | Downloads and parses GSE series matrix files into `GSEMatrix(Vec<ExpressionSet>)`. |
| `getGEO(..., GSEMatrix = FALSE)` | Implemented for tested cases | Downloads and parses full SOFT GSE into `GSE`. |
| `parseGSEMatrix` | Implemented for tested cases | Preserves expression shape, feature/sample names, phenotype data, feature data row names, annotation, and experiment metadata. |
| `GDS2eSet` | Implemented for tested cases | Produces native `ExpressionSet` with expression matrix, feature rows, phenotype data, annotation, and experiment metadata. |
| `GDS2MA` | Implemented for tested cases | Produces native `MAList` with matrix dimensions, row names, column names, targets, and notes. |
| `getRNASeqData` | Implemented for tested cases | Produces native `SummarizedExperiment` with `counts`, annotation, sample metadata shell, genome metadata, and timestamp. |

## Still Needs Work

These are not blocking the current ported test suite, but they matter for a
more complete replacement of the original Bioconductor behavior.

| Area | What remains |
| --- | --- |
| Full Bioconductor semantic parity | The Rust structs match GEOquery test expectations, but they do not replicate every method, validation rule, coercion, or edge behavior from Bioconductor classes. |
| Rich `AnnotatedDataFrame` metadata | Implemented for the current native data model and tested conversion paths. Future work only applies if the crate adds deeper optional GPL annotation joins beyond current test-suite parity. |
| GPL feature matching | The test suite validates dimensions and row names, but full GPL feature annotation matching should be expanded for downstream analysis use. |
| Characteristic parsing | GSE matrix parsing passes the ported tests, but characteristic key/value expansion should be hardened and tested with more malformed real-world records. |
| Typed storage | `DataFrame::typed_column` provides typed views, but storage is still string-backed. A future pass could add schema-aware storage or columnar arrays for performance. |
| Error taxonomy | Structured domain variants exist for known GEO-specific failures. Some low-level network and parser errors intentionally remain wrapped to preserve source context. |
| API naming cleanup | Core function aliases and struct accessor aliases now exist. R-compatible trait names remain for familiarity and backward compatibility with the port. |
| Documentation examples | Public URL helpers have doctests. More narrative Rustdoc could be added, but the core examples compile and pass. |
| Broader fixture coverage | Local compressed GSM, GPL, GSE matrix, and RNA-seq fixtures now cover core parser paths. Supplemental listing fixtures would require an HTTP mocking layer and are not needed for current parity. |
| Full GPL feature matching | Current implementation preserves feature row names and passes parity tests. Full optional GPL annotation joins are outside current R test coverage and would be a new feature expansion. |
| Characteristic parsing hardening | Implemented and locally tested for repeated `key: value` GSE matrix characteristics, including semicolon collapsing per sample. Live GSE characteristic tests pass. |

## Current Parity Claim

The current claim is **test-suite parity with the reference R tests**. All R
`testthat` files have Rust counterparts, and both the local and live ignored
Rust suites passed in the latest run.

The remaining rows above are intentionally framed as future expansion areas,
not blockers for parity with the current GEOquery tests. Exact method-for-method
Bioconductor compatibility is not a native Rust target because this crate does
not embed R or Bioconductor.
