use anyhow::{Context, Result};
use std::time::Duration;
use serde::{Serialize, Deserialize};
use chrono::prelude::{DateTime, Local};
use postgres::Client;

#[derive(Serialize, Deserialize, Debug)]
pub struct PostBuildHookRecord {
    pub drv_path: String,
    pub out_paths: Vec<String>,
}


#[derive(Debug)]
pub struct BuildRecord {
    pub name: String,
    pub drv_path: String,
    pub out_paths: Vec<String>,
    pub timestamp: DateTime<Local>,
    pub build_elapsed: Duration,
    pub instance_type: String,
    pub instance_id: String,
    pub pull_request_number: i64,
}

impl BuildRecord {
    pub fn insert(&self, client: &mut Client) -> Result<u64> {
        client.execute(
            "INSERT INTO nix_build (
                name,
                drv_path,
                ctime,
                build_elapsed,
                instance_type,
                instance_id,
                pull_request_number
            ) VALUES ($1, $2, $3, make_interval(secs => $4), $5, $6, $7)",
            &[
                &self.name,
                &self.drv_path,
                &self.timestamp,
                &self.build_elapsed.as_secs_f64(),
                &self.instance_type,
                &self.instance_id,
                &self.pull_request_number
            ],
        ).context("Failed to execute SQL")
    }
}

// https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/instance-identity-documents.html
#[derive(Deserialize, Debug)]
pub struct InstanceIdentity {
    #[serde(rename = "instanceType")]
    pub instance_type: String,

    #[serde(rename = "instanceId")]
    pub instance_id: String,
}
