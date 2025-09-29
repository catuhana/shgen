use wasm_bindgen::prelude::*;

pub mod search;

#[wasm_bindgen]
pub struct Config(shgen_config_model_core::Config);

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(keywords: Vec<String>, search: search::SearchConfig) -> Self {
        Self(shgen_config_model_core::Config {
            keywords,
            search: search.inner(),
        })
    }

    #[wasm_bindgen(getter)]
    pub fn keywords(&self) -> Vec<String> {
        self.0.keywords.to_vec()
    }

    #[wasm_bindgen(getter)]
    pub fn search(&self) -> search::SearchConfig {
        self.0.search.clone().into()
    }
}
