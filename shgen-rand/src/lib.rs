use rand::{RngCore, SeedableRng as _};
use rand_aes::Aes128Ctr64;
use rand_chacha::ChaCha8Rng;

#[allow(clippy::large_enum_variant)]
pub enum Rng {
    Aes128Ctr64(Aes128Ctr64),
    // https://eprint.iacr.org/2019/1492.pdf Section 5.3
    ChaCha8(ChaCha8Rng),
}

impl Rng {
    #[must_use]
    pub fn from_best_available() -> Self {
        if Self::is_aes_available() {
            Self::Aes128Ctr64(Aes128Ctr64::from_os_rng())
        } else {
            Self::ChaCha8(ChaCha8Rng::from_os_rng())
        }
    }

    fn is_aes_available() -> bool {
        #[cfg(target_arch = "x86_64")]
        {
            is_x86_feature_detected!("aes")
        }

        #[cfg(target_arch = "x86")]
        {
            is_x86_feature_detected!("sse2") && is_x86_feature_detected!("aes")
        }

        #[cfg(any(target_arch = "aarch64", target_arch = "arm64ec"))]
        {
            std::arch::is_aarch64_feature_detected!("aes")
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            false
        }
    }
}

impl RngCore for Rng {
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        match self {
            Self::Aes128Ctr64(rng) => rng.fill_bytes(dst),
            Self::ChaCha8(rng) => rng.fill_bytes(dst),
        }
    }

    fn next_u32(&mut self) -> u32 {
        match self {
            Self::Aes128Ctr64(rng) => rng.next_u32(),
            Self::ChaCha8(rng) => rng.next_u32(),
        }
    }

    fn next_u64(&mut self) -> u64 {
        match self {
            Self::Aes128Ctr64(rng) => rng.next_u64(),
            Self::ChaCha8(rng) => rng.next_u64(),
        }
    }
}
