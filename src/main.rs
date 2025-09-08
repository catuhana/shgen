use std::{
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use core_affinity::{self, CoreId};
use ed25519_dalek::SigningKey;
use keep_awake::KeepAwake;
use rand::RngCore;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

use crate::{
    config::Config,
    matcher::Matcher,
    openssh_format::{OpenSSHFormatter, OpenSSHPrivateKey, OpenSSHPublicKey},
};

mod config;
mod matcher;
mod openssh_format;

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

static FOUND_KEY: OnceLock<(OpenSSHPublicKey, OpenSSHPrivateKey)> = OnceLock::new();
static STOP_WORKERS: AtomicBool = AtomicBool::new(false);

fn main() {
    let config = Config::load(parse_config_path_arg()).expect("failed to load configuration");

    let mut keep_awake = if config.runtime.keep_awake {
        Some(KeepAwake::new("shgen is generating keys"))
    } else {
        None
    };

    let mut worker_handles = Vec::with_capacity(config.runtime.threads);
    for thread_id in 0..config.runtime.threads {
        let matcher = Matcher::new(config.keywords.clone(), config.search.clone());

        worker_handles.push(
            std::thread::Builder::new()
                .name(format!("worker-{thread_id}"))
                .spawn(move || {
                    if config.runtime.set_affinity
                        && !core_affinity::set_for_current(CoreId { id: thread_id })
                    {
                        eprintln!("Failed to set core affinity for thread {thread_id}");
                    }

                    worker(matcher)
                })
                .expect("failed to spawn worker thread"),
        );
    }

    // Kind of hacky I guess?
    if let Some(ref mut keep_awake) = keep_awake {
        keep_awake.prevent_sleep();
    }

    println!("{}", config.generate_config_overview());
    for handle in worker_handles {
        handle.join().unwrap();
    }

    if let Some((public_key, private_key)) = FOUND_KEY.get() {
        // TODO: Print stuff like which keywords matched, in which fields,
        // how long it took, which thread found it, etc.
        config.output.save_keys(public_key, private_key);
    }
}

fn worker(matcher: Matcher) {
    let mut thread_rng = rand::rng();
    // https://eprint.iacr.org/2019/1492.pdf Section 5.3
    let mut chacha8_rng = ChaCha8Rng::from_rng(&mut thread_rng);

    let mut secret_keys = [0u8; 32 * 8];
    while !STOP_WORKERS.load(Ordering::Acquire) {
        chacha8_rng.fill_bytes(&mut secret_keys);

        for secret_key in secret_keys.chunks_exact(32) {
            let signing_key = SigningKey::from_bytes(secret_key.try_into().unwrap());
            let mut formatter = OpenSSHFormatter::new(signing_key, &mut thread_rng);

            if let Some((public_key, private_key)) = matcher.search_matches(&mut formatter) {
                if FOUND_KEY.set((public_key, private_key)).is_ok() {
                    STOP_WORKERS.store(true, Ordering::Release);
                }

                return;
            }
        }
    }
}

fn parse_config_path_arg() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();

    for index in 0..args.len() {
        if args[index] == "--config" || args[index] == "-c" {
            if index + 1 < args.len() {
                return Some(PathBuf::from(&args[index + 1]));
            }

            eprintln!("Expected a path after '{}'", args[index]);
            std::process::exit(1);
        }
    }

    None
}
