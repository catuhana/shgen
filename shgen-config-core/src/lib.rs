pub mod search;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Config {
    pub keywords: Vec<String>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub search: search::Config,
}
