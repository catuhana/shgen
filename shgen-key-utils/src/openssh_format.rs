#![allow(clippy::cast_possible_truncation)]

use shgen_core::{OpenSSHPrivateKey, OpenSSHPublicKey};

use base64::{
    Engine,
    engine::{GeneralPurpose, general_purpose::STANDARD_NO_PAD},
};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SigningKey, VerifyingKey};
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};

pub struct OpenSSHFormatter<'a, R: Rng> {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,

    rng: &'a mut R,
    base64_engine: GeneralPurpose,
}

const SSH_KEY_ALGORITHM_NAME: &str = "ssh-ed25519";

const PRIVATE_KEY_MAGIC: &[u8] = b"openssh-key-v1\0";

const CIPHER_NAME: &[u8] = b"none";

const KDF_NAME: &[u8] = b"none";
const KDF_OPTIONS: &[u8] = b"";

const NUMBER_OF_KEYS: u32 = 1;

const PRIVATE_KEY_PADDING: u32 = (4 + 4) // Two 4 bytes check-ints
            + (4 + SSH_KEY_ALGORITHM_NAME.len() as u32) // Algorithm name length, algorithm name
            + (4 + PUBLIC_KEY_LENGTH as u32) // Public key length, public key
            + (4 + (PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH) as u32) // Private key length, public key + private key
            + 4; // Comment length, comment
const PRIVATE_KEY_PADDING_LEN: u32 = (8 - (PRIVATE_KEY_PADDING % 8)) % 8;

const PUBLIC_KEY_BLOB_SIZE: usize = 4 // Algorithm name length
        + SSH_KEY_ALGORITHM_NAME.len() // Algorithm name
        + 4 // Public key length
        + PUBLIC_KEY_LENGTH; // Public key
const PRIVATE_KEY_BLOB_SIZE: usize = PRIVATE_KEY_MAGIC.len() // Private key magic
    + (4 + CIPHER_NAME.len()) // Cipher name length, cipher name
    + (4 + KDF_NAME.len()) // KDF name length, KDF name
    + (4 + KDF_OPTIONS.len()) // KDF options length, KDF options
    + 4 // Number of keys length
    + (4 + PUBLIC_KEY_BLOB_SIZE) // Public key blob length
    + 4 // Private key section length
    + (4 + 4) // Two 4 bytes check-ints
    + (4 + SSH_KEY_ALGORITHM_NAME.len()) // Algorithm name length, algorithm name
    + (4 + PUBLIC_KEY_LENGTH) // Public key length, public key
    + (4 + (PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH)) // Private key length, public key + private key
    + 4 // Comment length, comment
    + PRIVATE_KEY_PADDING_LEN as usize; // Padding (1, 2, 3, ..., n), maximum 7 bytes

impl<'a, R: Rng> OpenSSHFormatter<'a, R> {
    pub fn new(signing_key: SigningKey, rng: &'a mut R) -> Self {
        let verifying_key: VerifyingKey = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
            rng,
            base64_engine: STANDARD_NO_PAD,
        }
    }

    #[must_use]
    pub fn format_public_key(&self) -> OpenSSHPublicKey {
        const OPENSSH_PUBLIC_KEY_LENGTH: usize = SSH_KEY_ALGORITHM_NAME.len() // Algorithm name length
            + 1 // Space (1 byte)
            + base64::encoded_len(PUBLIC_KEY_BLOB_SIZE, false).unwrap(); // Base64 encoded public key blob length

        let mut public_key = String::with_capacity(OPENSSH_PUBLIC_KEY_LENGTH);

        public_key.push_str(SSH_KEY_ALGORITHM_NAME);
        public_key.push(' ');
        self.base64_engine
            .encode_string(self.generate_public_key_blob(), &mut public_key);

        OpenSSHPublicKey::new(public_key)
    }

    pub fn format_private_key(&mut self) -> OpenSSHPrivateKey {
        const PRIVATE_KEY_HEADER: &str = "-----BEGIN OPENSSH PRIVATE KEY-----\n";
        const PRIVATE_KEY_FOOTER: &str = "-----END OPENSSH PRIVATE KEY-----\n";

        const PRIVATE_BLOB_ENCODED_LENGTH: usize =
            base64::encoded_len(PRIVATE_KEY_BLOB_SIZE, false).unwrap();

        const OPENSSH_PRIVATE_KEY_LENGTH: usize = PRIVATE_KEY_HEADER.len() // Private key header
            + PRIVATE_BLOB_ENCODED_LENGTH // Base64 encoded private key blob length
            + PRIVATE_BLOB_ENCODED_LENGTH / 70 // New lines (1 every 70 chars)
            + if PRIVATE_BLOB_ENCODED_LENGTH.is_multiple_of(70) { 0 } else { 1 } // Possible last new line
            + PRIVATE_KEY_FOOTER.len(); // Private key footer

        let mut private_key = String::with_capacity(OPENSSH_PRIVATE_KEY_LENGTH);

        private_key.push_str(PRIVATE_KEY_HEADER);

        let mut encoded_private_key_blob_buffer = [0u8; PRIVATE_BLOB_ENCODED_LENGTH];
        let private_key_blob = self.generate_private_key_blob();

        self.base64_engine
            .encode_slice(private_key_blob, &mut encoded_private_key_blob_buffer)
            .unwrap();
        for chunk in encoded_private_key_blob_buffer.chunks(70) {
            private_key.push_str(unsafe { std::str::from_utf8_unchecked(chunk) });
            private_key.push('\n');
        }

        private_key.push_str(PRIVATE_KEY_FOOTER);

        OpenSSHPrivateKey::new(private_key)
    }

    #[must_use]
    pub fn format_fingerprint(&self, fingerprint: &Fingerprint) -> String {
        let public_key_blob = self.generate_public_key_blob();
        match fingerprint {
            Fingerprint::Sha1 => self.base64_engine.encode(Sha1::digest(public_key_blob)),
            Fingerprint::Sha256 => self.base64_engine.encode(Sha256::digest(public_key_blob)),
            Fingerprint::Sha384 => self.base64_engine.encode(Sha384::digest(public_key_blob)),
            Fingerprint::Sha512 => self.base64_engine.encode(Sha512::digest(public_key_blob)),
        }
    }

    fn generate_public_key_blob(&self) -> [u8; PUBLIC_KEY_BLOB_SIZE] {
        let mut public_key_blob = [0u8; PUBLIC_KEY_BLOB_SIZE];
        let mut cursor = 0;

        let ssh_key_algorithm_name_len = SSH_KEY_ALGORITHM_NAME.len();

        // Write the algorithm name length
        public_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(ssh_key_algorithm_name_len as u32).to_be_bytes());
        cursor += 4;

        // Write the algorithm name
        public_key_blob[cursor..(cursor + ssh_key_algorithm_name_len)]
            .copy_from_slice(SSH_KEY_ALGORITHM_NAME.as_bytes());
        cursor += ssh_key_algorithm_name_len;

        // Write the public key length
        public_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(PUBLIC_KEY_LENGTH as u32).to_be_bytes());
        cursor += 4;

        // Write the public key
        public_key_blob[cursor..(cursor + PUBLIC_KEY_LENGTH)]
            .copy_from_slice(self.verifying_key.as_bytes());
        cursor += PUBLIC_KEY_LENGTH;

        debug_assert_eq!(cursor, PUBLIC_KEY_BLOB_SIZE);

        public_key_blob
    }

    fn generate_private_key_blob(&mut self) -> [u8; PRIVATE_KEY_BLOB_SIZE] {
        let mut private_key_blob = [0u8; PRIVATE_KEY_BLOB_SIZE];
        let mut cursor = 0;

        // Write the private key magic
        private_key_blob[cursor..(cursor + PRIVATE_KEY_MAGIC.len())]
            .copy_from_slice(PRIVATE_KEY_MAGIC);
        cursor += PRIVATE_KEY_MAGIC.len();

        // Write the cipher name length and cipher name
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(CIPHER_NAME.len() as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + CIPHER_NAME.len())].copy_from_slice(CIPHER_NAME);
        cursor += CIPHER_NAME.len();

        // Write the KDF name length, KDF name and KDF options length
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(KDF_NAME.len() as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + KDF_NAME.len())].copy_from_slice(KDF_NAME);
        cursor += KDF_NAME.len();
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(KDF_OPTIONS.len() as u32).to_be_bytes());
        cursor += 4;

        // Write the number of keys
        private_key_blob[cursor..(cursor + 4)].copy_from_slice(&NUMBER_OF_KEYS.to_be_bytes());
        cursor += 4;

        let public_key_blob = self.generate_public_key_blob();
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(public_key_blob.len() as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + public_key_blob.len())]
            .copy_from_slice(&public_key_blob);
        cursor += public_key_blob.len();

        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(PRIVATE_KEY_PADDING + PRIVATE_KEY_PADDING_LEN).to_be_bytes());
        cursor += 4;

        let mut checkint = [0u8; 4];
        self.rng.fill_bytes(&mut checkint);

        // Write two random check-ints
        private_key_blob[cursor..(cursor + 4)].copy_from_slice(&checkint);
        cursor += 4;
        private_key_blob[cursor..(cursor + 4)].copy_from_slice(&checkint);
        cursor += 4;

        // Write the algorithm name length, and algorithm name
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(SSH_KEY_ALGORITHM_NAME.len() as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + SSH_KEY_ALGORITHM_NAME.len())]
            .copy_from_slice(SSH_KEY_ALGORITHM_NAME.as_bytes());
        cursor += SSH_KEY_ALGORITHM_NAME.len();

        // Write the public key length, and public key
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&(PUBLIC_KEY_LENGTH as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + PUBLIC_KEY_LENGTH)]
            .copy_from_slice(self.verifying_key.as_bytes());
        cursor += PUBLIC_KEY_LENGTH;

        // Write the public + private key length, private key and public key
        private_key_blob[cursor..(cursor + 4)]
            .copy_from_slice(&((PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH) as u32).to_be_bytes());
        cursor += 4;
        private_key_blob[cursor..(cursor + SECRET_KEY_LENGTH)]
            .copy_from_slice(self.signing_key.as_bytes());
        cursor += SECRET_KEY_LENGTH;
        private_key_blob[cursor..(cursor + PUBLIC_KEY_LENGTH)]
            .copy_from_slice(self.verifying_key.as_bytes());
        cursor += PUBLIC_KEY_LENGTH;

        // Write the comment length and comment
        private_key_blob[cursor..(cursor + 4)].copy_from_slice(&0u32.to_be_bytes());
        cursor += 4;

        // Write padding
        for index in 1..=PRIVATE_KEY_PADDING_LEN {
            private_key_blob[cursor] = index as u8;
            cursor += 1;
        }

        debug_assert_eq!(cursor, private_key_blob.len());

        private_key_blob
    }
}

pub enum Fingerprint {
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}
