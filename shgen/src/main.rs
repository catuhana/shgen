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

use rand::RngCore as _;
use shgen_config_native::Config;
use shgen_keep_awake::KeepAwake;
use shgen_rand::Rng;
use shgen_types::{OpenSSHPrivateKey, OpenSSHPublicKey};

use ed25519_dalek::{SECRET_KEY_LENGTH, SigningKey};

use shgen_key_utils::{matcher::Matcher, openssh};

#[global_allocator]
static ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

static FOUND_KEY: OnceLock<(OpenSSHPublicKey, OpenSSHPrivateKey)> = OnceLock::new();

static KEYS_COUNTER: AtomicU64 = AtomicU64::new(0);
static STOP_WORKERS: AtomicBool = AtomicBool::new(false);

fn main() {
    let config = Config::load(parse_config_path_arg()).expect("failed to load configuration");

    let mut keep_awake = if config.runtime.keep_awake {
        match KeepAwake::new("shgen is generating keys") {
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

        let mut last_keys_count = 0u64;
        let mut last_check_time = Instant::now();

        while !STOP_WORKERS.load(Ordering::Relaxed) {
            // TODO: Maybe make this configurable
            std::thread::sleep(Duration::from_secs(2));

            let now = Instant::now();
            let elapsed_total = start_time.elapsed();
            let elapsed_since_last = now.duration_since(last_check_time);

            let generated_keys = KEYS_COUNTER.load(Ordering::Relaxed);

            let keys_since_last = generated_keys - last_keys_count;
            let instant_rate = keys_since_last as f64 / elapsed_since_last.as_secs_f64();

            let overall_rate = generated_keys as f64 / elapsed_total.as_secs_f64();

            let formatted_elapsed = format!(
                "{:02}:{:02}:{:02}",
                elapsed_total.as_secs() / 3600,
                (elapsed_total.as_secs() % 3600) / 60,
                elapsed_total.as_secs() % 60
            );

            print!(
                "\r\x1b[K[{formatted_elapsed}] {generated_keys} keys @ {overall_rate:.0} keys/s avg. @ {instant_rate:.0} keys/s inst."
            );
            let _ = std::io::stdout().flush();

            last_keys_count = generated_keys;
            last_check_time = now;
        }
    })
    .expect("failed to spawn status thread");

    if let Some(ref mut keep_awake) = keep_awake
        && let Err(error) = keep_awake.prevent_sleep()
    {
        eprintln!("Failed to prevent system sleep: {error}");
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
    const BATCH_COUNT: usize = (8 * 1024) / SECRET_KEY_LENGTH;

    let mut rng = Rng::from_best_available();
    let mut formatter = openssh::format::Formatter::empty();

    while !STOP_WORKERS.load(Ordering::Acquire) {
        let mut secret_keys_batch = [0u8; SECRET_KEY_LENGTH * BATCH_COUNT];
        rng.fill_bytes(&mut secret_keys_batch);

        let (secret_keys_chunks, _) = secret_keys_batch.as_chunks::<SECRET_KEY_LENGTH>();
        for secret_key in secret_keys_chunks {
            let signing_key = SigningKey::from_bytes(secret_key);
            formatter.update_keys(signing_key);

            if let Some((public_key, private_key)) =
                matcher.search_matches(&mut formatter, &mut rng)
            {
                if FOUND_KEY.set((public_key, private_key)).is_ok() {
                    STOP_WORKERS.store(true, Ordering::Release);
                }

                return;
            }
        }

        KEYS_COUNTER.fetch_add(BATCH_COUNT as u64, Ordering::Relaxed);
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
