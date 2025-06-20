/// Get an environment variable, or fail if it's not set
///
/// # Errors
///
/// - If the environment variable is not set or is non-Unicode
pub fn require_env<T: AsRef<str>>(name: T) -> anyhow::Result<String> {
    std::env::var(name.as_ref())
        .map_err(|_| anyhow::anyhow!("required environment variable `{}` not set", name.as_ref()))
}

/// Read a file from the specified environment variable, or fail if it's not set or doesn't exist
///
/// # Errors
///
/// - If the environment variable is not set or is non-Unicode
/// - If the file does not exist or is not readable (see [`std::fs::read`])
pub fn require_env_file<T: AsRef<str>>(name: T) -> anyhow::Result<Vec<u8>> {
    let path = require_env(name)?;
    std::fs::read(&path).map_err(|err| anyhow::anyhow!("failed to read `{path}`: {err}"))
}
