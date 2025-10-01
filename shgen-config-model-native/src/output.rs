use std::path::PathBuf;

#[derive(Debug, serde::Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Config {
    pub save_to: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_to: PathBuf::from("found-keys"),
        }
    }
}

#[cfg(feature = "fs")]
mod fs_impls {
    use super::Config;

    use shgen_core::{OpenSSHPrivateKey, OpenSSHPublicKey};

    impl Config {
        pub fn save_keys(&self, public_key: &OpenSSHPublicKey, private_key: &OpenSSHPrivateKey) {
            let save_dir = &self.save_to;

            std::fs::create_dir_all(save_dir).expect("failed to create output directory");

            let public_key_path = save_dir.join("id_ed25519.pub");
            let private_key_path = save_dir.join("id_ed25519");

            std::fs::write(&public_key_path, &**public_key).expect("failed to write public key");
            std::fs::write(&private_key_path, &**private_key).expect("failed to write private key");

            println!("Saved keys to {}", save_dir.display());
        }
    }
}
