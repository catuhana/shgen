use std::ops::Deref;

pub struct OpenSSHPublicKey(String);

impl OpenSSHPublicKey {
    pub fn new(key: String) -> Self {
        Self(key)
    }
}

impl Deref for OpenSSHPublicKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OpenSSHPrivateKey(String);

impl OpenSSHPrivateKey {
    pub fn new(key: String) -> Self {
        Self(key)
    }
}

impl Deref for OpenSSHPrivateKey {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
