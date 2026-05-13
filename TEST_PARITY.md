# GEOquery Test Parity

The Rust test suite now has coverage corresponding to every R `testthat` file
from the reference GEOquery package.

## Commands

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

## R Test Files Represented

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

## Latest Verification

Both commands were run successfully after the latest Rust implementation work:

```sh
cargo test
cargo test -- --ignored
```

The ignored suite requires network access to NCBI and downloads GEO files.
