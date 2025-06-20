//! Service utilities for working with S3 buckets

use crate::service::environment::require_env;
use anyhow::bail;
use s3::creds::Credentials;
use s3::{Bucket, Region};

/// Returns an S3 bucket with options from the environment
///
/// # Arguments
///
/// - `public`: whether the bucket will be used for presigning (using a public endpoint)
///
/// # Errors
///
/// - If some environment variables are missing
/// - If the bucket does not exist
pub async fn require_s3(public: bool) -> anyhow::Result<Bucket> {
    let region = Region::Custom {
        region: require_env("S3_REGION")?,
        endpoint: if public {
            require_env("S3_PUBLIC_ENDPOINT")?
        } else {
            require_env("S3_ENDPOINT")?
        },
    };

    let bucket = require_env("S3_BUCKET")?;
    let creds = Credentials::new(
        Some(&require_env("S3_ACCESS_KEY_ID")?),
        Some(&require_env("S3_SECRET_ACCESS_KEY")?),
        None,
        None,
        None,
    )?;

    let bucket = Bucket::new(&bucket, region, creds)?.with_path_style();

    if !bucket.exists().await? {
        bail!("s3 bucket does not exist");
    }

    Ok(*bucket)
}
