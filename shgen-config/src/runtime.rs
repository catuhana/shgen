use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub threads: usize,
    pub keep_awake: bool,
    pub set_affinity: bool,
}

impl Config {
    pub fn default() -> Self {
        Self {
            threads: Self::default_threads(),
            keep_awake: Self::default_keep_awake(),
            set_affinity: Self::default_set_affinity(),
        }
    }

    pub fn default_threads() -> usize {
        std::thread::available_parallelism().unwrap().get()
    }

    pub const fn default_keep_awake() -> bool {
        true
    }

    pub const fn default_set_affinity() -> bool {
        true
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::default()
    }
}
