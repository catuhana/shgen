#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]

mod generate_keys;

use generate_keys::generate;
use shgen_cli::{Cli, Command};
use shgen_config_native::Config;

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
    let cli = Cli::try_parse().unwrap();
    let config = Config::load(cli.config).unwrap();

    match cli.command {
        Command::Benchmark => unimplemented!("benchmark command is not yet implemented"),
        Command::Generate => generate(config),
    }
}
