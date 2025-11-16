use base64::{Engine, engine::general_purpose::STANDARD_NO_PAD};
use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH, SigningKey, VerifyingKey};
use rand::Rng;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha384, Sha512};
use shgen_types::{OpenSSHPrivateKey, OpenSSHPublicKey};

use crate::openssh::Fingerprint;

pub struct Formatter {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Formatter {
    #[must_use]
    pub fn new(signing_key: SigningKey) -> Self {
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    #[must_use]
    pub fn empty() -> Self {
        Self::new(SigningKey::from_bytes(&[0u8; SECRET_KEY_LENGTH]))
    }

    pub fn update_keys(&mut self, signing_key: SigningKey) {
        self.verifying_key = signing_key.verifying_key();
        self.signing_key = signing_key;
    }

    #[must_use]
    pub fn format_public_key(&self) -> OpenSSHPublicKey {
        let mut public_key = String::with_capacity(
            constants::ALGORITHM.len()
                + 1
                + base64::encoded_len(sizes::PUBLIC_KEY_BLOB, false).unwrap(),
        );

        public_key.push_str(constants::ALGORITHM);
        public_key.push(' ');
        STANDARD_NO_PAD.encode_string(self.build_public_key_blob(), &mut public_key);

        OpenSSHPublicKey::new(public_key)
    }

    pub fn format_private_key<R: Rng>(&mut self, rng: &mut R) -> OpenSSHPrivateKey {
        const HEADER: &str = "-----BEGIN OPENSSH PRIVATE KEY-----\n";
        const FOOTER: &str = "-----END OPENSSH PRIVATE KEY-----\n";
        const ENCODED_LEN: usize = base64::encoded_len(sizes::PRIVATE_KEY_BLOB, false).unwrap();
        const CAPACITY: usize = HEADER.len() + ENCODED_LEN + ENCODED_LEN / 70 + 1 + FOOTER.len();

        let mut private_key = String::with_capacity(CAPACITY);
        private_key.push_str(HEADER);

        let mut encoded_buffer = [0u8; ENCODED_LEN];
        let blob = self.build_private_key_blob(rng);

        STANDARD_NO_PAD
            .encode_slice(blob, &mut encoded_buffer)
            .unwrap();

        for chunk in encoded_buffer.chunks(70) {
            private_key.push_str(str::from_utf8(chunk).expect("base64 is not valid utf-8"));
            private_key.push('\n');
        }

        private_key.push_str(FOOTER);
        OpenSSHPrivateKey::new(private_key)
    }

    #[must_use]
    pub fn format_fingerprint(&self, fingerprint: &Fingerprint) -> String {
        let blob = self.build_public_key_blob();
        match fingerprint {
            Fingerprint::Sha1 => STANDARD_NO_PAD.encode(Sha1::digest(blob)),
            Fingerprint::Sha256 => STANDARD_NO_PAD.encode(Sha256::digest(blob)),
            Fingerprint::Sha384 => STANDARD_NO_PAD.encode(Sha384::digest(blob)),
            Fingerprint::Sha512 => STANDARD_NO_PAD.encode(Sha512::digest(blob)),
        }
    }

    fn build_public_key_blob(&self) -> [u8; sizes::PUBLIC_KEY_BLOB] {
        let mut blob = [0u8; sizes::PUBLIC_KEY_BLOB];
        let mut writer = SshEncoder::new(&mut blob);

        writer.write_string(constants::ALGORITHM.as_bytes());
        writer.write_u32(PUBLIC_KEY_LENGTH as u32);
        writer.write_bytes(self.verifying_key.as_bytes());

        blob
    }

    fn build_private_key_blob<R: Rng>(&self, rng: &mut R) -> [u8; sizes::PRIVATE_KEY_BLOB] {
        let mut blob = [0u8; sizes::PRIVATE_KEY_BLOB];
        let mut writer = SshEncoder::new(&mut blob);

        // header
        writer.write_bytes(constants::MAGIC);
        writer.write_string(constants::CIPHER);
        writer.write_string(constants::KDF);
        writer.write_u32(constants::KDF_OPTIONS.len() as u32);
        writer.write_u32(1); // number of keys

        // public key blob
        let public_blob = self.build_public_key_blob();
        writer.write_u32(public_blob.len() as u32);
        writer.write_bytes(&public_blob);

        // private key section
        writer.write_u32((sizes::PRIVATE_KEY_SECTION + sizes::PRIVATE_KEY_PADDING) as u32);

        let mut checkint = [0u8; 4];
        rng.fill(&mut checkint);
        writer.write_bytes(&checkint);
        writer.write_bytes(&checkint);

        writer.write_string(constants::ALGORITHM.as_bytes());
        writer.write_u32(PUBLIC_KEY_LENGTH as u32);
        writer.write_bytes(self.verifying_key.as_bytes());
        writer.write_u32((PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH) as u32);
        writer.write_bytes(self.signing_key.as_bytes());
        writer.write_bytes(self.verifying_key.as_bytes());
        writer.write_u32(0); // empty comment

        // padding
        for i in 1..=sizes::PRIVATE_KEY_PADDING {
            writer.write_bytes(&[i as u8]);
        }

        blob
    }
}

mod constants {
    pub const ALGORITHM: &str = "ssh-ed25519";

    pub const MAGIC: &[u8] = b"openssh-key-v1\0";
    pub const CIPHER: &[u8] = b"none";
    pub const KDF: &[u8] = b"none";
    pub const KDF_OPTIONS: &[u8] = b"";
}

mod sizes {
    use super::constants;
    use ed25519_dalek::{PUBLIC_KEY_LENGTH, SECRET_KEY_LENGTH};

    pub const PUBLIC_KEY_BLOB: usize = (4 + constants::ALGORITHM.len()) // algorithm name length + name
        + (4 + PUBLIC_KEY_LENGTH); // public key length + key

    pub const PRIVATE_KEY_SECTION: usize = (4 + 4) +    // two check-ints
        (4 + constants::ALGORITHM.len()) +                 // algorithm name length + name
        (4 + PUBLIC_KEY_LENGTH) +                       // public key length + key
        (4 + (PUBLIC_KEY_LENGTH + SECRET_KEY_LENGTH)) + // private key length + keys
        4; // comment length

    pub const PRIVATE_KEY_PADDING: usize = (8 - (PRIVATE_KEY_SECTION % 8)) % 8;

    pub const PRIVATE_KEY_BLOB: usize = constants::MAGIC.len() +
        (4 + constants::CIPHER.len()) +
        (4 + constants::KDF.len()) +
        (4 + constants::KDF_OPTIONS.len()) +
        4 +                                                // number of keys
        (4 + PUBLIC_KEY_BLOB) +                            // public key blob length + blob
        (4 + (PRIVATE_KEY_SECTION + PRIVATE_KEY_PADDING)); // private key section length;
}

struct SshEncoder<'a> {
    buffer: &'a mut [u8],
    cursor: usize,
}

impl<'a> SshEncoder<'a> {
    const fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, cursor: 0 }
    }

    fn write_u32(&mut self, value: u32) {
        self.buffer[self.cursor..self.cursor + 4].copy_from_slice(&value.to_be_bytes());
        self.cursor += 4;
    }

    fn write_bytes(&mut self, bytes: &[u8]) {
        self.buffer[self.cursor..self.cursor + bytes.len()].copy_from_slice(bytes);
        self.cursor += bytes.len();
    }

    fn write_string(&mut self, s: &[u8]) {
        self.write_u32(s.len() as u32);
        self.write_bytes(s);
    }
}
