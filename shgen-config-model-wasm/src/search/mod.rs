use wasm_bindgen::prelude::*;

pub mod matching;

// For some reason `wasm-bindgen` creates multiple
// `__wbg-config` modules if we name those structs
// as `Config`. Applies to `matching.rs` as well.
#[derive(Clone, Debug)]
#[wasm_bindgen]
pub struct SearchConfig(shgen_config_model_core::search::Config);

#[wasm_bindgen]
impl SearchConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(fields: Vec<SearchFields>, matching: matching::MatchingConfig) -> Self {
        Self(shgen_config_model_core::search::Config {
            fields: fields.into_iter().map(Into::into).collect(),
            matching: matching.inner(),
        })
    }
}

impl SearchConfig {
    #[must_use]
    pub fn inner(self) -> shgen_config_model_core::search::Config {
        self.0
    }
}

impl From<shgen_config_model_core::search::Config> for SearchConfig {
    fn from(config: shgen_config_model_core::search::Config) -> Self {
        Self(config)
    }
}

impl From<SearchFields> for shgen_config_model_core::search::SearchFields {
    fn from(field: SearchFields) -> Self {
        match field {
            SearchFields::PrivateKey => Self::PrivateKey,
            SearchFields::PublicKey => Self::PublicKey,
            SearchFields::Sha1Fingerprint => Self::Sha1Fingerprint,
            SearchFields::Sha256Fingerprint => Self::Sha256Fingerprint,
            SearchFields::Sha384Fingerprint => Self::Sha384Fingerprint,
            SearchFields::Sha512Fingerprint => Self::Sha512Fingerprint,
        }
    }
}

#[wasm_bindgen]
pub enum SearchFields {
    PrivateKey,
    PublicKey,
    Sha1Fingerprint,
    Sha256Fingerprint,
    Sha384Fingerprint,
    Sha512Fingerprint,
}
