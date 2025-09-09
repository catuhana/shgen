use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub threads: usize,
    pub keep_awake: bool,
    pub set_affinity: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            threads: std::thread::available_parallelism().unwrap().get(),
            keep_awake: true,
            set_affinity: true,
        }
    }
}
