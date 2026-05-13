---
layout: default
title: R GEOquery Mapping
description: Mapping from common R GEOquery workflows to geoquery-rs.
---

geoquery-rs intentionally keeps many R-compatible names while also adding idiomatic snake-case aliases for common Rust code.

| R GEOquery | geoquery-rs |
| --- | --- |
| `getGEO("GDS507")` | `get_geo(GetGeoOptions { GEO: Some("GDS507"), ..Default::default() })` |
| `getGEO("GSE2553", GSEMatrix = FALSE)` | `get_geo(GetGeoOptions { GEO: Some("GSE2553"), GSEMatrix: false, ..Default::default() })` |
| `parseGEO("file.soft.gz")` | `parse_geo("file.soft.gz", None, None, false, true)` |
| `Meta(x)` | `x.meta()` or `Meta::Meta(&x)` |
| `Table(x)` | `x.table()` or `Table::Table(&x)` |
| `Columns(x)` | `x.columns()` or `Columns::Columns(&x)` |
| `GSMList(gse)` | `gse.gsm_list()` or `GSMList::GSMList(&gse)` |
| `GPLList(gse)` | `gse.gpl_list()` or `GPLList::GPLList(&gse)` |
| `GDS2MA(gds)` | `gds_to_ma(&gds, do_log2)` or `GDS2MA(&gds, do_log2)` |
| `GDS2eSet(gds)` | `gds_to_expression_set(&gds, do_log2)` or `GDS2eSet(&gds, do_log2)` |
| `getGEOSuppFiles("GSE...")` | `getGEOSuppFiles("GSE...", make_directory, base_dir, fetch_files, filter_regex)` |
| `searchGEO("...")` | `searchGEO("...", retmax)` |

## Object replacements

| Bioconductor/R object | geoquery-rs type |
| --- | --- |
| `GDS`, `GPL`, `GSM`, `GSE` | `GDS`, `GPL`, `GSM`, `GSE` |
| `GEODataTable` | `GEODataTable` |
| `data.frame` | `DataFrame` |
| `ExpressionSet` | `ExpressionSet` |
| `MAList` | `MAList` |
| `SummarizedExperiment` | `SummarizedExperiment` |
| `AnnotatedDataFrame` | `AnnotatedDataFrame` |
| `MIAME` | `ExperimentData` |

## Compatibility boundary

geoquery-rs is a native Rust crate. It does not embed R, Bioconductor, or S4 semantics. The goal is practical GEOquery workflow parity with Rust types and error handling.
