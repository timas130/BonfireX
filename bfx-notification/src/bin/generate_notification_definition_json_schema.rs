use bfx_core::logging::setup_logging;
use bfx_notification::definition::NotificationDefinition;
use schemars::schema_for;
use std::env::current_dir;
use std::fs::File;
use std::io::Write;
use std::path;
use tracing::info;

fn main() -> anyhow::Result<()> {
    setup_logging();

    let schema = schema_for!(NotificationDefinition);

    let dir = env!("CARGO_MANIFEST_DIR");
    let path = path::PathBuf::from(dir).join("definition-schema.json");

    let mut file = File::create(&path)?;
    file.write_all(serde_json::to_string_pretty(&schema)?.as_bytes())?;

    let relative_path = path.strip_prefix(current_dir()?).unwrap_or(path.as_path());
    info!("schema written to {}", relative_path.display());

    Ok(())
}
