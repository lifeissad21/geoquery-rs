---
layout: default
title: Native Rust access to GEO
description: Native Rust tools for NCBI GEO data, inspired by R/Bioconductor GEOquery.
hero: true
eyebrow: NCBI GEO data in Rust
hero_title: Parse, download, and convert GEO datasets without an R runtime.
hero_text: geoquery-rs is a native Rust rewrite of the R/Bioconductor GEOquery package. It reads GEO SOFT files, series matrix files, metadata tables, RNA-seq quantification resources, and supplemental file listings.
---

## What the crate covers

| Area | Public API |
| --- | --- |
| Accession helpers | `url_for_accession`, `browse_geo_accession`, `browse_rnaseq_search` |
| Download and parse | `get_geo`, `getGEO`, `getGEOfile`, `parse_geo`, `parseGEO` |
| Local parsers | `parseGDS`, `parseGPL`, `parseGSM`, `parseGSE`, `parseGSEMatrix` |
| Conversions | `GDS2MA`, `GDS2eSet`, `gds_to_ma`, `gds_to_expression_set` |
| Supplemental files | `getGEOSuppFiles`, `getGEOSuppFileURL`, `getDirListing`, `getGEOSeriesFileListing` |
| RNA-seq resources | `getRNASeqData`, `getRNASeqQuantResults`, `readRNAQuantRawCounts`, `readRNAQuantAnnotation` |
| Search | `searchGEO`, `searchFieldsGEO` |

## Project documentation

- [Getting Started]({{ '/getting-started/' | relative_url }}) covers installation, CLI use, library use, and tests.
- [API Reference]({{ '/api/' | relative_url }}) lists the full public API by workflow.
- [Examples]({{ '/examples/' | relative_url }}) gives practical Rust snippets.
- [R GEOquery Mapping]({{ '/r-mapping/' | relative_url }}) maps familiar R calls to geoquery-rs.
- [Publishing]({{ '/publishing/' | relative_url }}) covers rustdoc, docs.rs, crates.io, and GitHub Pages.

This site explains project workflows and the public API at a high level. The generated Rust API docs are built by rustdoc and hosted on docs.rs after publishing.
