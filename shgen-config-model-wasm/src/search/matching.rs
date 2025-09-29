use wasm_bindgen::prelude::*;

#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct MatchingConfig(shgen_config_model_core::search::matching::Config);

#[wasm_bindgen]
impl MatchingConfig {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(all_keywords: bool, all_fields: bool) -> Self {
        Self(shgen_config_model_core::search::matching::Config {
            all_keywords,
            all_fields,
        })
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn all_keywords(&self) -> bool {
        self.0.all_keywords
    }

    #[wasm_bindgen(getter)]
    #[must_use]
    pub fn all_fields(&self) -> bool {
        self.0.all_fields
    }
}

impl MatchingConfig {
    #[must_use]
    pub fn inner(self) -> shgen_config_model_core::search::matching::Config {
        self.0
    }
}
