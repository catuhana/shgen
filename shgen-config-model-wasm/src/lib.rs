use wasm_bindgen::prelude::*;

mod macros;

use macros::core_to_wasm_wrapper;

use crate::macros::core_enum_to_wasm;

core_to_wasm_wrapper! {
    #[derive(Debug)]
    pub struct Config(shgen_config_model_core::Config);
    constructor(keywords: Vec<String>, search: SearchConfig) {
        Self(shgen_config_model_core::Config {
            keywords,
            search: search.into(),
        })
    }

    getters {
        #[must_use]
        keywords -> Vec<String> => |config| config.0.keywords.clone();
        #[must_use]
        search -> SearchConfig => |config| config.0.search.clone().into();
    }
}

core_to_wasm_wrapper! {
    #[derive(Debug)]
    pub struct SearchConfig(shgen_config_model_core::search::Config);
    constructor(fields: Vec<SearchFields>, matching: MatchingConfig) {
        Self(shgen_config_model_core::search::Config {
            fields: fields.into_iter().map(Into::into).collect(),
            matching: matching.into(),
        })
    }
}

core_to_wasm_wrapper! {
    #[derive(Debug)]
    pub struct MatchingConfig(shgen_config_model_core::search::matching::Config);
    constructor(all_keywords: bool, all_fields: bool) {
        Self(shgen_config_model_core::search::matching::Config {
            all_keywords,
            all_fields,
        })
    }
}

core_enum_to_wasm! {
    pub enum SearchFields => shgen_config_model_core::search::SearchFields {
        PrivateKey,
        PublicKey,
        Sha1Fingerprint,
        Sha256Fingerprint,
        Sha384Fingerprint,
        Sha512Fingerprint,
    }
}
