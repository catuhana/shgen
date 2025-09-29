use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub save_to: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_to: PathBuf::from("found-keys"),
        }
    }
}
