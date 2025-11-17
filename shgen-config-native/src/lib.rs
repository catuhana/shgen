pub mod output;
pub mod runtime;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub shared: shgen_config_core::Config,
    #[serde(default)]
    pub runtime: runtime::Config,
    #[serde(default)]
    pub output: output::Config,
}

#[cfg(feature = "fs")]
mod fs_impls {
    use super::Config;

    use std::path::PathBuf;

    use figment::providers::Format as _;
    use figment::{Figment, providers::Yaml};

    impl Config {
        pub fn load(config_path: PathBuf) -> Result<Self, Box<figment::Error>> {
            let config: Self = Figment::new().merge(Yaml::file(config_path)).extract()?;

            config.validate()?;
            Ok(config)
        }

        pub fn validate(&self) -> Result<(), Box<figment::Error>> {
            if self.output.save_to.exists() && !self.output.save_to.is_dir() {
                return Err(Box::new(figment::Error::from(
                    "Save path exists but is not a directory",
                )));
            }

            if self.runtime.threads == 0 {
                return Err(Box::new(figment::Error::from(
                    "Number of threads must be greater than 0",
                )));
            }

            if self.runtime.threads > 192 {
                return Err(Box::new(figment::Error::from(
                    "Number of threads must be less than or equal to 192",
                )));
            }

            if self.shared.keywords.is_empty() {
                return Err(Box::new(figment::Error::from(
                    "At least one keyword must be specified",
                )));
            }

            if self.shared.keywords.len() > 64 {
                return Err(Box::new(figment::Error::from(
                    "Number of keywords must be less than or equal to 64",
                )));
            }

            Ok(())
        }
    }
}
