use std::{path::PathBuf, str::FromStr};

use facet::Facet;

#[derive(Debug, Facet)]
pub struct Cli {
    #[facet(positional, default = Command::Generate)]
    pub command: Command,

    #[facet(named, short = 'c', default = PathBuf::from("config.yaml"))]
    pub config: PathBuf,
}

impl Cli {
    pub fn try_parse() -> Result<Self, Box<dyn std::error::Error>> {
        facet_args::from_std_args().map_err(|error| error.into())
    }
}

#[derive(Debug, Facet)]
#[repr(C)]
pub enum Command {
    Generate,
    Benchmark,
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "generate" => Ok(Self::Generate),
            "benchmark" => Ok(Self::Benchmark),
            _ => Err(format!("invalid command: {}", s)),
        }
    }
}
