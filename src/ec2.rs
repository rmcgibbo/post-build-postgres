use crate::structs::InstanceIdentity;
use anyhow::Result;
use std::fs;
use std::time::Duration;
use ureq;

const AWS_INSTANCE_IDENTITY_URL: &str =
    "http://169.254.169.254/latest/dynamic/instance-identity/document";

pub fn get_ec2_metadata() -> Result<InstanceIdentity> {
    if let Ok(true) = fs::read_to_string("/sys/devices/virtual/dmi/id/product_uuid").map(|s| s.starts_with("ec2")) {
        let response = ureq::get(AWS_INSTANCE_IDENTITY_URL)
            .timeout(Duration::from_millis(500))
            .call()?;
        let json: InstanceIdentity = response.into_json().unwrap();
        return Ok(json);
    }

    Ok(InstanceIdentity {
        instance_id: "".to_string(),
        instance_type: "".to_string(),
    })
}
