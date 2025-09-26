use ed25519_dalek::SigningKey;
use rand_chacha::{
    ChaCha8Rng,
    rand_core::{RngCore, SeedableRng},
};
use shgen_config_model::Config;
use shgen_keys::{matcher::Matcher, openssh_format::OpenSSHFormatter};
use wasm_bindgen::prelude::*;

// TODO: Mimalloc apparently supports WASM,
// experiment with it.

#[wasm_bindgen]
struct Generator {
    matcher: Matcher,
    rng: ChaCha8Rng,
}

#[wasm_bindgen]
impl Generator {
    #[wasm_bindgen(constructor)]
    pub fn new(config: JsValue) -> Self {
        let config: Config = serde_wasm_bindgen::from_value(config).unwrap();
        let matcher = Matcher::new(config.keywords, config.search);

        let mut seed = [0u8; 32];
        getrandom::fill(&mut seed).unwrap();

        Self {
            matcher,
            rng: ChaCha8Rng::from_seed(seed),
        }
    }

    #[wasm_bindgen]
    pub fn generate_batch(&mut self, batch_size: usize) -> JsValue {
        let mut secret_keys = vec![0u8; 32 * batch_size];
        self.rng.fill_bytes(&mut secret_keys);

        let (secret_keys_chunks, _) = secret_keys.as_chunks::<32>();
        for secret_key in secret_keys_chunks {
            let signing_key = SigningKey::from_bytes(secret_key);
            let mut formatter = OpenSSHFormatter::new(signing_key, &mut self.rng);

            if let Some((public_key, private_key)) = self.matcher.search_matches(&mut formatter) {
                return serde_wasm_bindgen::to_value(&(public_key, private_key)).unwrap();
            }
        }

        JsValue::NULL
    }
}

#[wasm_bindgen(start)]
fn start() {
    console_error_panic_hook::set_once();
}
