use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub all_keywords: bool,
    pub all_fields: bool,
}

impl Config {
    pub const fn default() -> Self {
        Self {
            all_keywords: Self::default_all_keywords(),
            all_fields: Self::default_all_fields(),
        }
    }

    pub const fn default_all_keywords() -> bool {
        false
    }

    pub const fn default_all_fields() -> bool {
        false
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}
