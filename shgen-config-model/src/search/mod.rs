use serde::Deserialize;

mod matching;

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    pub fields: Vec<SearchFields>,
    pub matching: matching::Config,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            fields: vec![SearchFields::PublicKey, SearchFields::Sha256Fingerprint],
            matching: matching::Config::default(),
        }
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
