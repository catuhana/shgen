use std::path::PathBuf;

use figment::{
    Figment,
    providers::{Format as _, Yaml},
};
use serde::Deserialize;

pub mod output;
pub mod runtime;
pub mod search;

#[derive(Deserialize)]
pub struct Config {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub search: search::Config,
    #[serde(default)]
    pub runtime: runtime::Config,
    #[serde(default)]
    pub output: output::Config,
}

impl Config {
    pub fn load(config_path: Option<PathBuf>) -> Result<Self, Box<figment::Error>> {
        let figment = Figment::new();

        let figment = if let Some(path) = config_path {
            figment.merge(Yaml::file(path))
        } else {
            figment
                .merge(Yaml::file("config.yaml"))
                .merge(Yaml::file("config.yml"))
        };

        figment.extract().map_err(Into::into)
    }

    pub fn generate_config_overview(&self) -> String {
        format!(
            "Keywords:\n  {:?}\n\
             Search fields:\n  {:?}\n\
             Matching:\n  All keywords: {}\n  All fields: {}\n\
             Threads: {}\n\
             Keep awake: {}\n\
             Set affinity: {}\n\
             Save folder: {}",
            self.keywords,
            self.search
                .fields
                .iter()
                .map(|field| format!("{field:?}"))
                .collect::<Vec<String>>(),
            self.search.matching.all_keywords,
            self.search.matching.all_fields,
            self.runtime.threads,
            self.runtime.keep_awake,
            self.runtime.set_affinity,
            self.output.save_to.display()
        )
    }
}
