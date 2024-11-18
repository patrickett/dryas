use dirs::config_dir;
use std::fs::File;

// TODO: replace this whole module with a Config struct
// so that we can have better interface for setting and getting properties

static CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("io error")]
    IoError(#[from] std::io::Error),
}

pub fn get_or_create() -> Result<File, ConfigError> {
    let mut config_path = config_dir().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Config directory not found",
    ))?;

    let app_name = env!("CARGO_PKG_NAME");

    // Add app name and config file name to the path
    config_path.push(app_name);
    std::fs::create_dir_all(&config_path)?; // Create the app config directory if it doesn't exist

    config_path.push(CONFIG_FILE_NAME);
    if !config_path.exists() {
        // Create the config file if it doesn't exist
        let file = std::fs::File::create(&config_path)?;
        Ok(file)
        // writeln!(file, "# Default configuration")?;
    } else {
        todo!()
    }
}
