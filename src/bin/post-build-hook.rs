use anyhow::{Context, Result};
use post_build_postgres::structs::PostBuildHookRecord;
use serde_json;
use std::env;
use systemd::journal;

fn main() -> Result<()> {
    let drv_path = env::var("DRV_PATH").context("DRV_PATH not set")?;
    let out_paths = env::var("OUT_PATHS")
        .context("OUT_PATHS not set")?
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    let record = PostBuildHookRecord {
        drv_path: drv_path,
        out_paths: out_paths,
    };

    print_journal(&serde_json::to_string(&record)?);
    Ok(())
}

fn print_journal(msg: &String) -> i32 {
    let lvl = 6;
    let identifier = "post-build-hook";
    journal::send(&[
        &format!("PRIORITY={}", lvl),
        &format!("MESSAGE={}", msg),
        &format!("SYSLOG_IDENTIFIER={}", identifier),
    ])
}
