use std::path::PathBuf;

use facet::Facet;

#[derive(Facet)]
pub struct Cli {
    #[facet(positional, default = PathBuf::from("config.yaml"))]
    pub config: PathBuf,
}

impl Cli {
    pub fn try_parse() -> Result<Self, Box<dyn std::error::Error>> {
        facet_args::from_std_args().map_err(|e| e.into())
    }
}
