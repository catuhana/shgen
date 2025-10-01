use ed25519_dalek::SigningKey;
use rand_chacha::{
    ChaCha8Rng,
    rand_core::{RngCore, SeedableRng},
};
use shgen_config_wasm::{Config, MatchingConfig, SearchConfig, SearchFields};
use shgen_key_utils::{matcher::Matcher, openssh};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Generator {
    matcher: Matcher,
    rng: ChaCha8Rng,
}

#[wasm_bindgen]
impl Generator {
    #[wasm_bindgen(constructor)]
    #[must_use]
    pub fn new(
        keywords: Vec<String>,
        fields: Vec<SearchFields>,
        all_keywords: bool,
        all_fields: bool,
    ) -> Self {
        let matching_config = MatchingConfig::new(all_keywords, all_fields);
        let search_config = SearchConfig::new(fields, matching_config);
        let config = Config::new(keywords, search_config);

        let matcher = Matcher::new(config.keywords(), config.search().into());

        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).unwrap();

        Self {
            matcher,
            rng: ChaCha8Rng::from_seed(seed),
        }
    }

    #[wasm_bindgen(js_name = generateBatch)]
    pub fn generate_batch(&mut self, batch_size: usize) -> JsValue {
        let mut secret_key = [0u8; 32];

        for _ in 0..batch_size {
            self.rng.fill_bytes(&mut secret_key);

            let signing_key = SigningKey::from_bytes(&secret_key);
            let mut formatter = openssh::format::Formatter::new(signing_key);

            if let Some((public_key, private_key)) =
                self.matcher.search_matches(&mut formatter, &mut self.rng)
            {
                return js_sys::Array::of2(
                    &JsValue::from_str(&public_key),
                    &JsValue::from_str(&private_key),
                )
                .into();
            }
        }

        JsValue::NULL
    }
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}
