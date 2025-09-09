use serde::Deserialize;

mod matching;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub fields: Vec<SearchFields>,
    pub matching: matching::Config,
}

impl Config {
    pub fn default() -> Self {
        Self {
            fields: Self::default_fields(),
            matching: Self::default_matching(),
        }
    }

    pub fn default_fields() -> Vec<SearchFields> {
        vec![SearchFields::PublicKey, SearchFields::Sha256Fingerprint]
    }

    pub const fn default_matching() -> matching::Config {
        matching::Config {
            all_keywords: false,
            all_fields: false,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SearchFields {
    PrivateKey,
    PublicKey,
    Sha1Fingerprint,
    Sha256Fingerprint,
    Sha384Fingerprint,
    Sha512Fingerprint,
}
