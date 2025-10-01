pub mod output;
pub mod runtime;

#[derive(Debug, serde::Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub shared: shgen_config_model_core::Config,
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
        pub fn load(config_path: Option<PathBuf>) -> Result<Self, Box<figment::Error>> {
            Figment::new()
                .merge(if let Some(path) = config_path {
                    Yaml::file(path)
                } else if std::fs::exists("config.yaml").is_ok_and(|exists| exists) {
                    Yaml::file("config.yaml")
                } else if std::fs::exists("config.yml").is_ok_and(|exists| exists) {
                    Yaml::file("config.yml")
                } else {
                    return Err(Box::new(figment::Error::from(
                        "No configuration file found, tried config.yaml and config.yml",
                    )));
                })
                .extract()
                .map_err(Into::into)
        }

        #[must_use]
        pub fn generate_config_overview(&self) -> String {
            format!(
                "Keywords:\n  {:?}\n\
             Search fields:\n  {:?}\n\
             Matching:\n  All keywords: {}\n  All fields: {}\n\
             Threads: {}\n\
             Keep awake: {}\n\
             Set affinity: {}\n\
             Save folder: {}",
                self.shared.keywords,
                self.shared
                    .search
                    .fields
                    .iter()
                    .map(|field| format!("{field:?}"))
                    .collect::<Vec<String>>(),
                self.shared.search.matching.all_keywords,
                self.shared.search.matching.all_fields,
                self.runtime.threads,
                self.runtime.keep_awake,
                self.runtime.pin_threads,
                self.output.save_to.display()
            )
        }
    }
}
