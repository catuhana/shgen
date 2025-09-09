use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub all_keywords: bool,
    pub all_fields: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            all_keywords: false,
            all_fields: true,
        }
    }
}
