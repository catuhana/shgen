use serde::Deserialize;

pub mod output;
pub mod runtime;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub shared: shgen_config_model_core::Config,
    #[serde(default)]
    pub runtime: runtime::Config,
    #[serde(default)]
    pub output: output::Config,
}
