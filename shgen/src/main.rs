#![allow(clippy::cast_precision_loss)]

use std::{
    io::Write,
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use shgen_config_fs::{ConfigExt as _, output::ConfigExt as _};
use shgen_config_model_native::Config;
use shgen_core::{OpenSSHPrivateKey, OpenSSHPublicKey};
use shgen_keep_awake::KeepAwake;

use ed25519_dalek::SigningKey;
use rand::RngCore;
use rand_chacha::{ChaCha8Rng, rand_core::SeedableRng};

use shgen_keys::{matcher::Matcher, openssh_format::OpenSSHFormatter};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

static FOUND_KEY: OnceLock<(OpenSSHPublicKey, OpenSSHPrivateKey)> = OnceLock::new();

static KEYS_COUNTER: AtomicU64 = AtomicU64::new(0);
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
        let matcher = Matcher::new(config.shared.keywords.clone(), config.shared.search.clone());

        worker_handles.push(
            std::thread::Builder::new()
                .name(format!("worker-{thread_id}"))
                .spawn(move || {
                    if config.runtime.pin_threads {
                        gdt_cpus::pin_thread_to_core(thread_id).unwrap_or_else(|e| {
                            eprintln!("Failed to set core affinity for thread {thread_id}: {e}");
                        });
                    }

                    worker(&matcher);
                })
                .expect("failed to spawn worker thread"),
        );
    }

    let status_thread = std::thread::Builder::new()
        .name("status-report".to_string())
        .spawn(|| {
            let start_time = Instant::now();

            while !STOP_WORKERS.load(Ordering::Relaxed) {
                // TODO: Maybe make this configurable
                std::thread::sleep(Duration::from_secs(2));

                let elapsed = start_time.elapsed();

                let generated_keys = KEYS_COUNTER.load(Ordering::Relaxed);
                let overall_rate = generated_keys as f64 / elapsed.as_secs_f64();

                let formatted_elapsed = format!(
                    "{:02}:{:02}:{:02}",
                    elapsed.as_secs() / 3600,
                    (elapsed.as_secs() % 3600) / 60,
                    elapsed.as_secs() % 60
                );
                print!(
                    "\r\x1b[K[{formatted_elapsed}] {generated_keys} keys generated @ {overall_rate:.0} keys/s average"
                );
                let _ = std::io::stdout().flush();
            }
        })
        .expect("failed to spawn status thread");

    // Kind of hacky I guess?
    if let Some(ref mut keep_awake) = keep_awake {
        keep_awake.prevent_sleep();
    }

    println!("{}", config.generate_config_overview());

    status_thread.join().unwrap();
    for handle in worker_handles {
        handle.join().unwrap();
    }

    if let Some((public_key, private_key)) = FOUND_KEY.get() {
        // TODO: Print stuff like which keywords matched, in which fields,
        // how long it took, which thread found it, etc.
        config.output.save_keys(public_key, private_key);
    }
}

fn worker(matcher: &Matcher) {
    // TODO: Experiment with different batch
    // sizes, maybe even make it configurable.
    const BATCH_SIZE: usize = 8;

    let mut thread_rng = rand::rng();
    // https://eprint.iacr.org/2019/1492.pdf Section 5.3
    let mut chacha8_rng = ChaCha8Rng::from_rng(&mut thread_rng);

    let mut secret_keys = [0u8; 32 * BATCH_SIZE];
    while !STOP_WORKERS.load(Ordering::Relaxed) {
        chacha8_rng.fill_bytes(&mut secret_keys);

        // There can't be any remainders, so discard it.
        let (secret_keys_chunks, _) = secret_keys.as_chunks::<32>();
        for secret_key in secret_keys_chunks {
            let signing_key = SigningKey::from_bytes(secret_key);
            let mut formatter = OpenSSHFormatter::new(signing_key, &mut thread_rng);

            if let Some((public_key, private_key)) = matcher.search_matches(&mut formatter) {
                if FOUND_KEY.set((public_key, private_key)).is_ok() {
                    STOP_WORKERS.store(true, Ordering::Release);
                }

                return;
            }
        }

        KEYS_COUNTER.fetch_add(BATCH_SIZE as u64, Ordering::Relaxed);
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
