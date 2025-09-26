#![cfg(not(feature = "wasm"))]

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub threads: usize,
    pub keep_awake: bool,
    pub pin_threads: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: std::thread::available_parallelism()
                .map(std::num::NonZeroUsize::get)
                .unwrap_or(1),
            keep_awake: true,
            pin_threads: true,
        }
    }
}
