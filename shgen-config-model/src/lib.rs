use serde::Deserialize;

pub mod output;
pub mod runtime;
pub mod search;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub search: search::Config,
    #[cfg(not(feature = "wasm-js"))]
    #[serde(default)]
    pub runtime: runtime::Config,
    #[cfg(not(feature = "wasm-js"))]
    #[serde(default)]
    pub output: output::Config,
}
