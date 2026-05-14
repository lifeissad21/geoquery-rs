use geoquery::{getGEO, GEOObject, GetGeoOptions, Meta, Table};
use std::env;

fn main() -> geoquery::Result<()> {
    let accession = env::args().nth(1).unwrap_or_else(|| "GDS507".to_string());
    let destdir = env::temp_dir().join("geoquery-rust-smoke-test");

    println!("Fetching {accession} into {}", destdir.display());

    let geo = getGEO(GetGeoOptions {
        GEO: Some(&accession),
        destdir,
        GSEMatrix: false,
        ..Default::default()
    })?;

    match geo {
        GEOObject::GDS(gds) => {
            print_metadata("GDS", gds.Meta());
            print_table_head(gds.Table(), 5);
        }
        GEOObject::GPL(gpl) => {
            print_metadata("GPL", gpl.Meta());
            print_table_head(gpl.Table(), 5);
        }
        GEOObject::GSM(gsm) => {
            print_metadata("GSM", gsm.Meta());
            print_table_head(gsm.Table(), 5);
        }
        GEOObject::GSE(gse) => {
            print_metadata("GSE", gse.Meta());
            println!("GSM records: {}", gse.gsms.len());
            println!("GPL records: {}", gse.gpls.len());
        }
        GEOObject::GSEMatrix(esets) => {
            println!("GSE matrix files parsed: {}", esets.len());
            if let Some(eset) = esets.first() {
                println!(
                    "Expression matrix: {} rows x {} columns",
                    eset.exprs.nrow(),
                    eset.exprs.ncol()
                );
            }
        }
    }

    Ok(())
}

fn print_metadata(kind: &str, metadata: &geoquery::classes::Header) {
    println!("\nObject type: {kind}");
    for key in ["geo_accession", "title", "platform", "type"] {
        if let Some(values) = metadata.get(key) {
            println!("{key}: {}", values.join("; "));
        }
    }
}

fn print_table_head(table: &geoquery::classes::DataFrame, rows: usize) {
    println!("\nTable: {} rows x {} columns", table.nrow(), table.ncol());
    if table.ncol() == 0 {
        println!("No data table found.");
        return;
    }

    println!("{}", table.column_names().join("\t"));
    for row_idx in 0..table.nrow().min(rows) {
        let values = table
            .row_values(row_idx)
            .into_iter()
            .map(|value| value.unwrap_or_else(|| "NA".to_string()))
            .collect::<Vec<_>>();
        println!("{}", values.join("\t"));
    }
}
