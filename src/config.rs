use std::path::PathBuf;

use figment::{
    Figment,
    providers::{Format, Yaml},
};
use serde::Deserialize;

use crate::openssh_format::{OpenSSHPrivateKey, OpenSSHPublicKey};

#[derive(Deserialize)]
pub struct Config {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub search: SearchConfig,
    #[serde(default)]
    pub runtime: RuntimeConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

impl Config {
    pub fn load(config_path: Option<PathBuf>) -> Result<Self, Box<figment::Error>> {
        let figment = Figment::new();

        let figment = if let Some(path) = config_path {
            figment.merge(Yaml::file(path))
        } else {
            figment
                .merge(Yaml::file("config.yaml"))
                .merge(Yaml::file("config.yml"))
        };

        figment.extract().map_err(Into::into)
    }

    pub fn generate_config_overview(&self) -> String {
        format!(
            "Keywords:\n  {:?}\n\
             Search fields:\n  {:?}\n\
             Matching:\n  All keywords: {}\n  All fields: {}\n\
             Threads: {}\n\
             Keep awake: {}\n\
             Set affinity: {}\n\
             Save folder: {}\n",
            self.keywords,
            self.search
                .fields
                .iter()
                .map(|field| format!("{field:?}"))
                .collect::<Vec<String>>(),
            self.search.matching.all_keywords,
            self.search.matching.all_fields,
            self.runtime.threads,
            self.runtime.keep_awake,
            self.runtime.set_affinity,
            self.output.save_to.display()
        )
    }
}

#[derive(Clone, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub fields: Vec<SearchFields>,
    pub matching: MatchingConfig,
}

impl SearchConfig {
    pub fn default() -> Self {
        Self {
            fields: Self::default_fields(),
            matching: Self::default_matching(),
        }
    }

    pub fn default_fields() -> Vec<SearchFields> {
        vec![SearchFields::PublicKey, SearchFields::Sha256Fingerprint]
    }

    pub const fn default_matching() -> MatchingConfig {
        MatchingConfig {
            all_keywords: false,
            all_fields: false,
        }
    }
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RuntimeConfig {
    pub threads: usize,
    pub keep_awake: bool,
    pub set_affinity: bool,
}

impl RuntimeConfig {
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

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct OutputConfig {
    pub save_to: PathBuf,
}

impl OutputConfig {
    pub fn save_keys(&self, public_key: &OpenSSHPublicKey, private_key: &OpenSSHPrivateKey) {
        let save_dir = &self.save_to;

        std::fs::create_dir_all(save_dir).expect("failed to create output directory");

        let public_key_path = save_dir.join("id_ed25519.pub");
        let private_key_path = save_dir.join("id_ed25519");

        std::fs::write(&public_key_path, public_key).expect("failed to write public key");
        std::fs::write(&private_key_path, private_key).expect("failed to write private key");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut private_key_permissions = std::fs::metadata(&private_key_path)
                .expect("failed to get metadata for private key")
                .permissions();
            permissions.set_mode(0o600);

            let mut public_key_permissions = std::fs::metadata(&public_key_path)
                .expect("failed to get metadata for public key")
                .permissions();
            permissions.set_mode(0o644);

            std::fs::set_permissions(&private_key_path, permissions)
                .expect("failed to set permissions for private key");
            std::fs::set_permissions(&public_key_path, permissions)
                .expect("failed to set permissions for public key");
        }

        println!("Saved keys to {}", save_dir.display());
    }

    pub fn default() -> Self {
        Self {
            save_to: Self::default_save_to(),
        }
    }

    pub fn default_save_to() -> PathBuf {
        PathBuf::from("found-keys")
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Clone, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct MatchingConfig {
    pub all_keywords: bool,
    pub all_fields: bool,
}

impl MatchingConfig {
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

impl Default for MatchingConfig {
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
