---
layout: default
title: API Reference
description: Public functions, structs, enums, and traits in geoquery-rs.
---

This page documents the public API surface by workflow. For exact Rust signatures, trait impls, and generated source links, use [docs.rs](https://docs.rs/geoquery-rs).

## High-level entry points

### `get_geo` / `getGEO`

Download and parse a GEO accession, or parse a supplied local file.

```rust
pub fn get_geo(options: GetGeoOptions<'_>) -> Result<GEOObject>
pub fn getGEO(options: GetGeoOptions<'_>) -> Result<GEOObject>
```

Use `get_geo` for idiomatic Rust. Use `getGEO` when porting R GEOquery-style code. GSE accessions default to series matrix parsing through `GetGeoOptions::GSEMatrix = true`.

### `GetGeoOptions`

Options for `get_geo` and `getGEO`.

| Field | Meaning |
| --- | --- |
| `GEO` | GEO accession to download, such as `GDS507` or `GSE2553`. |
| `filename` | Local SOFT or matrix file to parse instead of downloading. |
| `destdir` | Directory used for downloads. Defaults to the system temp directory. |
| `GSElimits` | Optional inclusive sample bounds for full GSE SOFT parsing. |
| `GSEMatrix` | For GSE accessions, parse series matrix files when true. |
| `AnnotGPL` | Prefer platform annotation files when downloading GPL data. |
| `getGPL` | Fetch platform records referenced by GSE records when supported. |
| `parseCharacteristics` | Expand GSE matrix sample characteristics when supported. |

### `GEOObject`

Return enum for high-level parsers and downloaders.

```rust
pub enum GEOObject {
    GDS(GDS),
    GPL(GPL),
    GSM(GSM),
    GSE(GSE),
    GSEMatrix(Vec<ExpressionSet>),
}
```

## URL helpers

| Function | Purpose |
| --- | --- |
| `url_for_accession(geo)` | Return the NCBI GEO browser URL for an accession. |
| `urlForAccession(geo)` | R-compatible alias for `url_for_accession`. |
| `browse_geo_accession(geo)` | Return the GEO browser URL. This crate returns the URL instead of opening a browser. |
| `browseGEOAccession(geo)` | R-compatible alias for `browse_geo_accession`. |
| `browse_rnaseq_search()` | Return the GEO RNA-seq counts search URL. |
| `browseWebsiteRNASeqSearch()` | R-compatible alias for `browse_rnaseq_search`. |

## Download and parse functions

| Function | Purpose |
| --- | --- |
| `getGEOfile(GEO, destdir, AnnotGPL, amount)` | Download a GEO SOFT or GPL annotation file and return the local path. |
| `get_geo_file(geo, destdir, annot_gpl, amount)` | Snake-case alias for `getGEOfile`. |
| `getAndParseGSEMatrices(GEO, destdir)` | Download and parse all GSE series matrix files. |
| `downloadFile(url, destfile, mode_binary)` | Download a URL into a local file. |
| `parse_geo(...)` | Snake-case alias for local GEO parsing. |
| `parseGEO(...)` | Infer and parse a local GEO SOFT or matrix file. |
| `parse_gse(fname, gse_limits)` | Snake-case alias for full GSE SOFT parsing. |
| `parse_gse_matrix(fname)` | Snake-case alias for GSE series matrix parsing. |

## Local parsers

| Function | Return type | Input |
| --- | --- | --- |
| `parseGDS(fname)` | `Result<GDS>` | GEO dataset SOFT file. |
| `parseGPL(fname)` | `Result<GPL>` | GEO platform SOFT or annotation file. |
| `parseGSM(fname)` | `Result<GSM>` | GEO sample SOFT file. |
| `parseGSE(fname, GSElimits)` | `Result<GSE>` | Full GEO series SOFT file. |
| `parseGSEMatrix(fname)` | `Result<ExpressionSet>` | GSE series matrix file. |

## Data model

### GEO records

| Type | Description |
| --- | --- |
| `GEOData` | Base GEO record shape with metadata and a `GEODataTable`. |
| `GDS` | GEO dataset record with metadata, optional platform, and table data. |
| `GPL` | GEO platform record. |
| `GSM` | GEO sample record. |
| `GSE` | GEO series record containing `gsms` and `gpls` maps. |
| `GEODataTable` | Pair of column descriptions and main table data. |

### Tables and matrices

| Type | Description |
| --- | --- |
| `Header` | `BTreeMap<String, Vec<String>>` for GEO metadata. |
| `DataFrame` | Schema-aware typed table with optional row names and column metadata. |
| `ColumnData` | Internal typed column storage: `Int`, `Float`, `Text`, or `Mixed`. |
| `Value` | Scalar value used in mixed typed columns. |
| `TypedColumn` | Typed view over a `DataFrame` column: `Integer`, `Float`, or `Text`. |
| `NumericMatrix` | Row-major matrix with optional `f64` values and row/column names. |

### Analysis objects

| Type | Description |
| --- | --- |
| `ExpressionSet` | Native replacement for Bioconductor `ExpressionSet`. |
| `MAList` | Native replacement for limma `MAList`. |
| `SummarizedExperiment` | Native replacement for Bioconductor `SummarizedExperiment`. |
| `AnnotatedDataFrame` | Native replacement for Bioconductor `AnnotatedDataFrame`. |
| `ExperimentData` | Structured experiment metadata derived from GEO headers. |
| `GenomeInfo` | Genome build metadata for RNA-seq quantification files. |
| `GEOMatrix` | Compatibility alias for `ExpressionSet`. |

## Data access methods

| Type | Methods |
| --- | --- |
| `DataFrame` | `new`, `with_row_names`, `empty_with_row_names`, `empty`, `nrow`, `ncol`, `column_index`, `column_names`, `get`, `get_by_index`, `row_values`, `column_values`, `row_name`, `typed_column`, `column_data`, `set_column_metadata` |
| `NumericMatrix` | `new`, `nrow`, `ncol`, `get`, `row_name`, `column_name` |
| `GDS`, `GPL`, `GSM`, `GEOData` | `meta`, `data_table`, `table`, `columns`, `accession` |
| `GSE` | `meta`, `gsm_list`, `gpl_list` |
| `GEODataTable` | `columns`, `table` |
| `ExpressionSet`, `MAList`, `SummarizedExperiment`, `AnnotatedDataFrame` | `nrow`, `ncol` |
| `SummarizedExperiment` | `assay` |
| `ExperimentData` | `from_header`, `expinfo` |

## R-compatible accessor traits

| Trait | Method | Purpose |
| --- | --- | --- |
| `Meta` | `Meta()` | Return parsed GEO metadata. |
| `Accession` | `Accession()` | Return accession metadata. |
| `dataTable` | `dataTable()` | Return combined table object. |
| `Columns` | `Columns()` | Return column descriptions. |
| `Table` | `Table()` | Return main table. |
| `GPLList` | `GPLList()` | Return GSE platforms. |
| `GSMList` | `GSMList()` | Return GSE samples. |

## Conversions

| Function | Purpose |
| --- | --- |
| `GDS2MA(gds, do_log2)` | Convert a `GDS` to `MAList`. |
| `gds_to_ma(gds, do_log2)` | Snake-case alias for `GDS2MA`. |
| `GDS2eSet(gds, do_log2)` | Convert a `GDS` to `ExpressionSet`. |
| `gds_to_expression_set(gds, do_log2)` | Snake-case alias for `GDS2eSet`. |
| `join_gpl_annotations(expression, gpl)` | Join GPL platform annotations into `ExpressionSet.featureData` by probe ID. |
| `join_gpl_annotations_by_key(expression, gpl, key)` | Join GPL annotations using `AnnotationKey`. |
| `AnnotationKey` | Annotation join key: `ProbeId`, `GeneSymbol`, `Entrez`, or `Custom(String)`. |

## Supplemental files

| Function or type | Purpose |
| --- | --- |
| `SuppFile` | Metadata for a supplemental file: `fname`, `url`, `destdir`, `filepath`, `GEO`. |
| `getGEOSuppFileURL(GEO)` | Return supplemental-file base URL. |
| `get_geo_supp_file_url(geo)` | Snake-case alias for `getGEOSuppFileURL`. |
| `getDirListing(url)` | Parse a GEO directory listing. |
| `getGEOSuppFiles(GEO, makeDirectory, baseDir, fetch_files, filter_regex)` | List or download supplemental files. |
| `getGEOSeriesFileListing(GSE)` | Return the tab-delimited series file listing. |

## RNA-seq helpers

| Function or type | Purpose |
| --- | --- |
| `RNASeqQuantResults` | Raw counts plus annotation table. |
| `getGSEDownloadPageURLs(gse)` | Return links from the GEO download page. |
| `getRNAQuantRawCountsURL(links)` | Select the raw-counts URL from download links. |
| `getRNAQuantAnnotationURL(links)` | Select the annotation URL from download links. |
| `extractFilenameFromDownloadURL(url)` | Extract the `file` query parameter. |
| `urlExtractRNASeqQuantGenomeInfo(url)` | Extract genome build, species, and filename from a URL. |
| `getRNASeqQuantGenomeInfo(gse)` | Return genome metadata for a GSE. |
| `readRNAQuantAnnotation(link)` | Read annotation TSV/TSV.GZ into `DataFrame`. |
| `readRNAQuantRawCounts(link)` | Read raw counts TSV/TSV.GZ into `NumericMatrix`. |
| `getRNASeqQuantResults(gse)` | Fetch raw counts and annotation. |
| `getRNASeqData(gse)` | Build a `SummarizedExperiment`. |
| `get_rna_seq_data(gse)` | Snake-case alias for `getRNASeqData`. |
| `hasRNASeqQuantifications(accession)` | Test whether a GSE exposes raw-count files. |

## Search

| Function | Purpose |
| --- | --- |
| `searchGEO(query, step)` | Search NCBI GEO DataSets with an Entrez query. |
| `searchFieldsGEO()` | Return searchable GEO DataSets field metadata. |

## Errors

Most functions return `geoquery::Result<T>`, an alias for `std::result::Result<T, GeoError>`.

| `GeoError` variant | Meaning |
| --- | --- |
| `UnsupportedAccession` | Accession prefix is not supported. |
| `InvalidAccession` | Accession failed validation. |
| `MissingFile` | Local file does not exist. |
| `UnknownEntity` | Parser could not infer GEO entity type. |
| `EntityMismatch` | Parser expected one entity but found another. |
| `NoMatrixFiles` | GSE matrix directory had no matrix files. |
| `MissingMatrix` | A matrix was required but absent. |
| `NoSupplementalFiles` | Supplemental directory had no matching files. |
| `MissingSupplemental` | A supplemental resource was required but absent. |
| `MissingField` | Required GEO metadata field was absent. |
| `InvalidCharacteristic` | Strict characteristic parsing context identified invalid input. |
| `GPLJoinFailure` | GPL annotation joining failed. |
| `AnnotationMissing` | A requested annotation column was missing. |
| `DuplicateFeature` | A duplicate feature was encountered where uniqueness was required. |
| `InvalidSOFTStructure` | SOFT structure was invalid. |
| `InvalidGSEMatrix` | GSE matrix structure was invalid. |
| `InvalidRNASeqCounts` | RNA-seq counts structure was invalid. |
| `NetworkFailure` | Network failure with additional GEO context and source error. |
| `ParseFailure` | Parse failure with preserved source context. |
| `Parse` | Text/table parser failure. |
| `Network` | HTTP failure from `reqwest`. |
| `Io` | Filesystem I/O failure. |
| `Csv` | TSV/CSV parser failure. |
| `Url` | URL parser failure. |
