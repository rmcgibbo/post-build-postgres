use anyhow::Result;
use humantime::parse_duration;
use post_build_postgres::journal::iter_builds_forever;
use postgres::{Client, NoTls};
use std::time::SystemTime;
use structopt::StructOpt;

fn parse_time_ago(s: &str) -> Result<SystemTime> {
    let now = SystemTime::now();
    if s.trim().len() == 0 || s.trim() == "now" {
        Ok(now)
    } else {
        let dur = parse_duration(&s.replace("ago", ""))?;
        Ok(now - dur)
    }
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "post-build-upload",
    about = "Service that uploads nix-build metadata"
)]
/// This is a service that uploads metadata about each invokation of nix-build
/// to postgres
///
/// It integrates with the post-build-hook, which records the data into the
/// systemd journal, and also records the relevant nixpkgs-review pull request
/// by gathering additional info from the journal that you need to make sure to
/// log.
///
struct Opt {
    /// Time to start from, e.g. '1 hr ago'.
    #[structopt(short, long, parse(try_from_str = parse_time_ago), default_value = "now")]
    since: SystemTime,

    /// Don't upload to postgres
    #[structopt(short, long)]
    dry_run: bool,

    #[structopt(long, env, hide_env_values = true)]
    database_url: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let mut client = Client::connect(&opt.database_url, NoTls)?;

    for x in iter_builds_forever(Some(opt.since))? {
        match x {
            Ok(build) => {
                if !opt.dry_run {
                    build.insert(&mut client)?;
                    println!("Insert {}", build.name);
                } else {
                    println!("Skipping insert (--dry-run) {:#?}", build);
                }
            }
            Err(err) => {
                eprintln!("{:#?}", err);
            }
        }
    }

    Ok(())
}
