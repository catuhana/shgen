use std::sync::atomic::{AtomicBool, Ordering};

use ed25519_dalek::{SECRET_KEY_LENGTH, SigningKey};
use rand::RngCore as _;
use shgen_config_native::Config;
use shgen_key_utils::{matcher::Matcher, openssh::format::Formatter};
use shgen_rand::Rng;
use shgen_types::{OpenSSHPrivateKey, OpenSSHPublicKey};

static STOP_WORKERS: AtomicBool = AtomicBool::new(false);

pub fn generate(config: Config) {
    let mut keep_awake = if config.runtime.keep_awake {
        match shgen_keep_awake::KeepAwake::new("shgen is generating keys") {
            Ok(guard) => Some(guard),
            Err(error) => {
                eprintln!("Could not keep awake: {error}");
                None
            }
        }
    } else {
        None
    };

    let mut worker_handles = Vec::with_capacity(config.runtime.threads);
    for thread_id in 0..config.runtime.threads {
        let matcher = Matcher::new(config.shared.keywords.clone(), config.shared.search.clone());

        worker_handles.push(
            std::thread::Builder::new()
                .name(format!("worker-{thread_id}"))
                .spawn(move || worker(&matcher))
                .expect("failed to spawn worker thread"),
        );
    }

    if let Some(ref mut keep_awake) = keep_awake
        && let Err(error) = keep_awake.prevent_sleep()
    {
        eprintln!("Failed to prevent system sleep: {error}");
    }

    let mut found_key: Option<(OpenSSHPublicKey, OpenSSHPrivateKey)> = None;
    for handle in worker_handles {
        if let Ok(Some(key_pair)) = handle.join()
            && found_key.is_none()
        {
            found_key = Some(key_pair);
            STOP_WORKERS.store(true, Ordering::Release);
        }
    }

    if let Some((public_key, private_key)) = found_key {
        config.output.save_keys(&public_key, &private_key);
    }
}

fn worker(matcher: &Matcher) -> Option<(OpenSSHPublicKey, OpenSSHPrivateKey)> {
    const BATCH_COUNT: usize = (8 * 1024) / SECRET_KEY_LENGTH;

    let mut rng = Rng::from_best_available();
    let mut formatter = Formatter::empty();

    let mut secret_keys_batch = [0u8; SECRET_KEY_LENGTH * BATCH_COUNT];
    while !STOP_WORKERS.load(Ordering::Acquire) {
        rng.fill_bytes(&mut secret_keys_batch);

        let (secret_keys_chunks, _) = secret_keys_batch.as_chunks::<SECRET_KEY_LENGTH>();
        for secret_key in secret_keys_chunks {
            let signing_key = SigningKey::from_bytes(secret_key);
            formatter.update_keys(signing_key);

            if let Some((public_key, private_key)) =
                matcher.search_matches(&mut formatter, &mut rng)
            {
                STOP_WORKERS.store(true, Ordering::Release);
                return Some((public_key, private_key));
            }
        }
    }

    None
}
