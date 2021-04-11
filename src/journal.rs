use crate::ec2::get_ec2_metadata;
use crate::structs::{BuildRecord, InstanceIdentity, PostBuildHookRecord};
use anyhow::{anyhow, Context, Result};
use chrono::prelude::{DateTime, Local};
use std::{path::Path, time::Duration, time::SystemTime, time::UNIX_EPOCH};
use systemd::{journal, Journal, JournalRecord};

pub fn iter_builds_forever(
    since: Option<SystemTime>,
) -> Result<impl Iterator<Item = Result<BuildRecord>>> {
    let ec2_metadata = get_ec2_metadata()?;

    let mut journal: Journal = journal::OpenOptions::default()
        .open()
        .context("Unable to open journal")?;

    journal.match_add("SYSLOG_IDENTIFIER", "post-build-hook")?;
    journal.match_add("SYSLOG_IDENTIFIER", "nixpkgs-review-start")?;
    journal.match_or()?;
    if let Some(time) = since {
        let usecs = time.duration_since(UNIX_EPOCH)?.as_micros();
        journal.seek_realtime_usec(usecs as u64)?;
    }

    let mut pr_number: Option<i64> = None;

    Ok(std::iter::from_fn(move || match journal.next_entry() {
        Ok(Some(entry)) => match entry.get("SYSLOG_IDENTIFIER") {
            Some(x) if x == "post-build-hook" => {
                // The outer Some tells from_fn to continue iterating,
                // the inner Some indicates that this iteration of the loop
                // yielded a result, and is filtered out by the filter_map.
                Some(Some(format(&journal, &entry, &ec2_metadata, &pr_number)))
            }
            Some(x) if x == "nixpkgs-review-start" => {
                match parse_nixpkgs_review_start_message(&entry) {
                    Ok(pr) => {
                        pr_number = Some(pr);
                        Some(None)
                    }
                    Err(err) => Some(Some(Err(err))),
                }
            }
            _ => Some(Some(Err(anyhow!("Unrecognized entry={:#?}", entry)))),
        },
        Ok(None) => {
            // No new events, so wait for a new event to appear in the journal
            // and then repeat.
            journal.wait(None).unwrap();
            Some(None)
        }
        _ => panic!("Journal failure?"),
    })
    // Filter out the steps from this iterator that return nothing, like when
    // we're just waiting for new events in the journal
    .filter_map(|x: Option<Result<BuildRecord>>| x))
}

fn parse_nixpkgs_review_start_message(entry: &JournalRecord) -> Result<i64> {
    let m: serde_json::Value = serde_json::from_str(entry.get("MESSAGE").ok_or(anyhow!("sdf"))?)?;
    let pr_number = m
        .get("pr")
        .ok_or(anyhow!("No 'pr' field"))?
        .as_i64()
        .ok_or(anyhow!("Type error in 'pr' field"))?;
    Ok(pr_number)
}

fn format(
    journal: &Journal,
    entry: &JournalRecord,
    ec2_metadata: &InstanceIdentity,
    pr_number: &Option<i64>,
) -> Result<BuildRecord> {
    let ts = journal.timestamp().unwrap();
    let dt: DateTime<Local> = ts.into();

    let msgs: &str = entry.get("MESSAGE").context("Failed to get MESSAGE")?;
    let msg: PostBuildHookRecord =
        serde_json::from_str(msgs).context("Unable to parse MESSAGE as json")?;
    let build_time = get_build_time(&msg.drv_path)?;
    let name = msg
        .drv_path
        .split("/")
        .last()
        .ok_or(anyhow!("Failed to split on /: {:?}", msg))?
        .splitn(2, "-")
        .last()
        .ok_or(anyhow!("No '-' in string: {:?}", msg))?
        .rsplitn(2, ".")
        .last()
        .ok_or(anyhow!("No extension: {:?}", msg))?;

    Ok(BuildRecord {
        name: name.to_string(),
        drv_path: msg.drv_path,
        out_paths: msg.out_paths,

        instance_id: ec2_metadata.instance_id.clone(),
        instance_type: ec2_metadata.instance_type.clone(),

        pull_request_number: pr_number.ok_or(anyhow!("missing pr num"))?,
        timestamp: dt,

        build_elapsed: build_time,
    })
}

// fn iso8601(st: std::time::SystemTime) -> String {
//     let dt: DateTime<Local> = st.into();
//     format!("{}", dt.format("%+"))
// }

fn get_log_path(drv_path: &str) -> Result<String> {
    let path = Path::new(drv_path);
    let base = path
        .components()
        .last()
        .context("Cannot extract path components")?
        .as_os_str()
        .to_os_string()
        .into_string()
        .map_err(|_| anyhow!("Cannot format drv_path as UTF-8"))?;

    let prefix = "/nix/var/log/nix/drvs/";
    let joined = vec![
        prefix.to_string(),
        base[0..2].to_string(),
        base[2..].to_string() + ".bz2",
    ]
    .join("/");
    Ok(joined.to_string())
}

fn get_build_time(drv_path: &str) -> Result<Duration> {
    let log_path = get_log_path(drv_path)?;
    let metadata = Path::new(&log_path)
        .metadata()
        .context("Cannot stat file")?;
    metadata
        .modified()?
        .duration_since(metadata.created()?)
        .context("Failed to extra time difference")
}
